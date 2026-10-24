"""Guard experiment integrity when tasks mutate only authorized fixture files."""

import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("benchmark", Path(__file__).with_name("agent-explorability.py"))
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)
STUDY_LAYOUT = "a63b71f"
# The frozen study protocols run gpt-6.1-sol at xhigh; launches must name both.
ACK = "gpt-6.1-sol:xhigh"
SCORE_SPEC = importlib.util.spec_from_file_location("scoring", Path(__file__).with_name("score_agent_explorability.py"))
SCORING = importlib.util.module_from_spec(SCORE_SPEC)
SCORE_SPEC.loader.exec_module(SCORING)


class ExperimentTests(unittest.TestCase):
    def setUp(self):
        fixture = tempfile.TemporaryDirectory()
        self.addCleanup(fixture.cleanup)
        source = Path(fixture.name)
        # The treatments patch the study-era CLI, whose help lived in src/main.rs;
        # a63b71f is the last revision with that layout, so later refactors of
        # the current tree cannot invalidate these checks.
        archive = subprocess.run(
            ["git", "archive", STUDY_LAYOUT, "src", "docs/guides", "Cargo.toml", "Cargo.lock"],
            cwd=BENCH.REPO, capture_output=True, check=True).stdout
        subprocess.run(["tar", "-x", "-C", str(source)], input=archive, check=True)
        shutil.copytree(BENCH.REPO / "scripts", source / "scripts")
        # Historical treatments need pre-adoption help; tests/integration/guides.rs guards the adopted wording.
        main = source / "src/main.rs"
        addition = "".join((BENCH.TASK.parent / name).read_text() for name in ["help-routing.txt", "help-recipe.txt"])
        escaped = addition.replace('\\', '\\\\').replace('"', '\\"')
        main.write_text(main.read_text().replace(escaped, ""))
        for mock in [patch.object(BENCH, "REPO", source),
                     patch.object(BENCH.subprocess, "check_output", return_value="synthetic-pre-adoption-fixture\n")]:
            mock.start()
            self.addCleanup(mock.stop)

    def test_historical_treatments_reject_help_that_already_contains_the_candidate(self):
        BENCH.with_help_recipe(BENCH.REPO, routing=True)
        for treatment in ["without-guides", "help-recipe", "help-routing"]:
            with self.subTest(treatment=treatment):
                destination = BENCH.REPO / treatment
                with self.assertRaisesRegex(ValueError, "pre-adoption"):
                    BENCH.prepare(destination, treatment=treatment)
                self.assertFalse(destination.exists())

    def test_supplied_binaries_keep_repository_snapshot_separate_from_arm_sources(self):
        BENCH.with_help_recipe(BENCH.REPO, routing=True)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            task = json.loads(BENCH.TASK.read_text())
            task["user_defaults"] = 'version = 1\n[scope]\nexclude = ["private/**"]\n'
            task_path = root / "task.json"
            task_path.write_text(json.dumps(task))
            pair = root / "pair"
            provenance = {arm: {"source_revision": f"source-{arm}"} for arm in "AB"}
            with patch.object(BENCH, "without_guides") as remove, patch.object(BENCH, "with_help_recipe") as recipe:
                BENCH.prepare(pair, task_path, binaries, "supplied-binaries", arm_provenance=provenance)
            remove.assert_not_called()
            recipe.assert_not_called()
            manifest = json.loads((pair / "manifest.json").read_text())
            self.assertNotIn("source_revision", manifest)
            self.assertNotIn("source_hashes", manifest)
            self.assertEqual(manifest["repository_snapshot"]["revision"], "synthetic-pre-adoption-fixture")
            self.assertEqual(manifest["arms"], {
                arm: {**provenance[arm], "binary_sha256": BENCH.sha(binary)}
                for arm, binary in zip("AB", binaries)
            })
            for arm in "AB":
                self.assertEqual((pair / arm / "target/debug/locmetry").read_text(), arm)
                self.assertFalse((pair / arm / "src").exists())
            for run in ["run-01", "run-02"]:
                self.assertEqual((pair / "user-config" / run / "locmetry/config.toml").read_text(), task["user_defaults"])
                self.assertFalse((pair / run / "user-config").exists())

    def test_pairs_share_each_frozen_binary_instead_of_copying_it(self):
        # 24 per-pair copies made the 0.8.0 regression run's evidence 905 MB (2026-10-22).
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.with_name("regression-0.8-correction-study.json").read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            destination = root / "campaign"
            with patch.object(full.BENCH, "REPO", BENCH.REPO):
                full.campaign(destination, binaries, ["control-revision", "candidate-revision"], protocol_path, prepare_only=True)
            for arm in "AB":
                frozen = destination / "binaries" / arm
                # Read-only, so no pair can change the file every pair shares.
                self.assertEqual(frozen.stat().st_mode & 0o222, 0)
                for pair in destination.glob("task*-rep*"):
                    self.assertEqual((pair / arm / "target/debug/locmetry").stat().st_ino, frozen.stat().st_ino)
            self.assertEqual(binaries[0].read_text(), "A")
            self.assertTrue(binaries[0].stat().st_mode & 0o200, "the supplied source stays writable")

    def test_pair_binaries_fall_back_to_copies_where_hard_links_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            pair = root / "pair"
            with patch.object(BENCH.os, "link", side_effect=OSError("cross-device link")):
                BENCH.prepare(pair, BENCH.TASK, binaries, "supplied-binaries")
            for arm, binary in zip("AB", binaries):
                target = pair / arm / "target/debug/locmetry"
                self.assertEqual(target.read_text(), arm)
                self.assertNotEqual(target.stat().st_ino, binary.stat().st_ino)

    def test_supplied_mode_requires_exactly_two_existing_binaries_before_creating_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("binary")
            for index, binaries in enumerate([None, [binary], [binary] * 3, [binary, root / "missing"]]):
                pair = root / f"pair-{index}"
                with self.subTest(binaries=binaries), self.assertRaises((ValueError, FileNotFoundError)):
                    BENCH.prepare(pair, binaries=binaries, treatment="supplied-binaries")
                self.assertFalse(pair.exists())

    def test_full_help_protocol_balances_each_task_and_keeps_discovery_rubrics_separate(self):
        full = self.full_help_runner()
        protocol = json.loads(full.PROTOCOL.read_text())
        from collections import Counter
        self.assertEqual(protocol["task_ids"], [1, 2, 3, 4, 5, 6, 7, 9, 10, 11])
        self.assertEqual(len(protocol["pairs"]), 40)
        self.assertEqual(protocol["concurrent_pairs"], 3)
        for number in protocol["task_ids"]:
            selected = [p for p in protocol["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {1, 2, 3, 4})
        old = json.loads(BENCH.TASK.with_name("task7.json").read_text())
        new = json.loads(BENCH.TASK.with_name("task7-full-help.json").read_text())
        self.assertEqual(new["oracle"]["shared_capabilities"], old["oracle"]["shared_capabilities"])
        self.assertEqual(new["oracle"]["coverage_rubric"], old["oracle"]["coverage_rubric"])
        self.assertIn("supplemental_semantics", new["oracle"])
        for phrase in ["all-counted", "Code", "zero", "0, 1, and 2"]:
            self.assertIn(phrase, new["prompt"])

    def full_help_runner(self, *, catalog=True):
        spec = importlib.util.spec_from_file_location("full_help", Path(__file__).with_name("full-help-study.py"))
        full = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(full)
        if catalog:
            # Unit tests must not depend on the installed Codex; the catalog check has its own tests.
            full.codex_catalog = lambda model, effort: {"codex_version": "codex-cli test", "model": model,
                                                         "efforts": [effort], "newer_in_family": []}
        return full

    def test_launch_requires_the_owner_to_acknowledge_the_frozen_model_and_effort(self):
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.with_name("regression-0.8-correction-study.json").read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            destination = root / "campaign"
            with patch.object(full.BENCH, "REPO", BENCH.REPO), patch.object(full.BENCH, "run_agent") as run:
                full.campaign(destination, binaries, ["control-revision", "candidate-revision"], protocol_path, prepare_only=True)
                self.assertEqual(json.loads((destination / "campaign.json").read_text())["codex_catalog"]["model"], "gpt-6.1-sol")
                for wrong in [None, "gpt-6.1-sol:high", "gpt-6-sol:xhigh"]:
                    with self.subTest(acknowledgement=wrong), self.assertRaisesRegex(ValueError, "acknowledge"):
                        full.run_frozen(destination, wrong)
                run.assert_not_called()
                self.assertFalse((destination / "started.json").exists())
                full.run_frozen(destination, ACK)
                started = json.loads((destination / "started.json").read_text())
                self.assertEqual(started["acknowledged"], ACK)
                self.assertEqual(started["codex_catalog"]["model"], "gpt-6.1-sol")
                self.assertEqual(run.call_count, 8)

    def test_codex_catalog_must_list_the_model_with_its_effort_and_names_newer_family_models(self):
        full = self.full_help_runner(catalog=False)
        models = {"models": [
            {"slug": "gpt-6.1-sol", "supported_reasoning_levels": [{"effort": "high"}, {"effort": "xhigh"}]},
            {"slug": "gpt-6.2-sol", "supported_reasoning_levels": [{"effort": "xhigh"}]},
            {"slug": "gpt-6.3-luna", "supported_reasoning_levels": [{"effort": "xhigh"}]},
            {"slug": "gpt-5.6-sol", "supported_reasoning_levels": [{"effort": "xhigh"}]},
        ]}

        def codex(args, **kwargs):
            output = "codex-cli 9.9.9\n" if args[1] == "--version" else json.dumps(models)
            return subprocess.CompletedProcess(args, 0, stdout=output, stderr="")

        with patch.object(full.subprocess, "run", side_effect=codex):
            entry = full.codex_catalog("gpt-6.1-sol", "xhigh")
            self.assertEqual(entry["codex_version"], "codex-cli 9.9.9")
            self.assertEqual(entry["newer_in_family"], ["gpt-6.2-sol"])
            with self.assertRaisesRegex(ValueError, "does not list"):
                full.codex_catalog("gpt-7-sol", "xhigh")
            with self.assertRaisesRegex(ValueError, "effort"):
                full.codex_catalog("gpt-6.1-sol", "max")

    def test_freezing_refuses_a_model_codex_does_not_offer_before_creating_output(self):
        full = self.full_help_runner(catalog=False)

        def unavailable(model, effort):
            raise ValueError(f"Codex does not list {model}")

        full.codex_catalog = unavailable
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.with_name("regression-0.8-correction-study.json").read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            with self.assertRaisesRegex(ValueError, "does not list"):
                full.campaign(root / "campaign", binaries, ["control-revision", "candidate"], protocol_path, prepare_only=True)
            self.assertFalse((root / "campaign").exists())

    def test_comparison_correction_protocol_preserves_gates_and_balances_four_tasks(self):
        from collections import Counter
        full = self.full_help_runner()
        original = json.loads(full.PROTOCOL.read_text())
        protocol = json.loads(full.PROTOCOL.with_name("compare-help-routing-study.json").read_text())
        self.assertEqual(protocol["task_ids"], [1, 4, 5, 7])
        self.assertEqual(len(protocol["pairs"]), 16)
        self.assertEqual(protocol["maximum_sessions"], 32)
        self.assertFalse(protocol["previous_trials_included"])
        for key in ["control", "model", "effort", "limits", "repetitions", "concurrent_pairs"]:
            self.assertEqual(protocol[key], original[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], original["decision_rule"][key])
        for number in protocol["task_ids"]:
            selected = [p for p in protocol["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {1, 2, 3, 4})

    def test_regression_protocol_reuses_full_help_gates_for_the_desk_review_tasks(self):
        from collections import Counter
        full = self.full_help_runner()
        original = json.loads(full.PROTOCOL.read_text())
        correction = json.loads(full.PROTOCOL.with_name("compare-help-routing-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-0.8-study.json").read_text())
        # Tasks 3, 4 and 8 are the ones the 2026-10-21 desk review flagged.
        self.assertEqual(protocol["task_ids"], [3, 4, 8])
        self.assertEqual(len(protocol["pairs"]), 12)
        self.assertEqual(protocol["maximum_sessions"], 24)
        self.assertFalse(protocol["previous_trials_included"])
        # The control is the last checked candidate, the comparison-routing correction.
        self.assertEqual(protocol["control"]["source_revision"], "c9098e20a3dade541c61ca8ca2bc347943e672be")
        self.assertEqual(protocol["control"]["binary_sha256"],
                         "882fe74a64f6df405f47deaa9cfc6f88190bbd337f4c7531183e36b926f35a05")
        self.assertEqual(protocol["task_files"], {})
        for key in ["model", "effort", "limits", "repetitions", "concurrent_pairs"]:
            self.assertEqual(protocol[key], original[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], correction["decision_rule"][key])
        for number in protocol["task_ids"]:
            selected = [p for p in protocol["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {1, 2, 3, 4})
            task = json.loads(BENCH.TASK.with_name(f"task{number}.json").read_text())
            self.assertEqual(task["limits"], protocol["limits"])

    def test_regression_correction_reruns_only_the_failed_comparison_task(self):
        from collections import Counter
        full = self.full_help_runner()
        regression = json.loads(full.PROTOCOL.with_name("regression-0.8-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-0.8-correction-study.json").read_text())
        # Task 4 alone failed its call gate on 2026-10-22; its control stays fixed.
        self.assertEqual(protocol["task_ids"], [4])
        self.assertEqual(protocol["maximum_sessions"], 8)
        self.assertFalse(protocol["previous_trials_included"])
        for key in ["control", "model", "effort", "limits", "repetitions", "concurrent_pairs", "task_files"]:
            self.assertEqual(protocol[key], regression[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], regression["decision_rule"][key])
        self.assertEqual(Counter(p["order"] for p in protocol["pairs"]), {"AB": 2, "BA": 2})
        self.assertEqual({p["repetition"] for p in protocol["pairs"]}, {1, 2, 3, 4})
        full.validate_protocol(protocol)

    def test_category_correction_reruns_only_the_naming_task_with_the_single_task_call_allowance(self):
        from collections import Counter
        full = self.full_help_runner()
        regression = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-correction-study.json").read_text())
        # Task 12 alone failed its uncached-input gate on 2026-10-24; its control stays fixed.
        self.assertEqual(protocol["task_ids"], [12])
        self.assertEqual(protocol["maximum_sessions"], 8)
        self.assertFalse(protocol["previous_trials_included"])
        for key in ["control", "model", "effort", "limits", "repetitions", "concurrent_pairs", "task_files"]:
            self.assertEqual(protocol[key], regression[key])
        for key in ["correctness", "per_task_medians", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], regression["decision_rule"][key])
        # A single-task correction allows one call above the control's total
        # (owner decision of 2026-10-22, docs/agent-regression-checks.md).
        totals = protocol["decision_rule"]["overall_all_run_totals"]
        self.assertEqual(totals["agent_tool_calls"], {"maximum_absolute_increase": 1})
        self.assertEqual(totals["uncached_input_tokens"], regression["decision_rule"]["overall_all_run_totals"]["uncached_input_tokens"])
        self.assertEqual(Counter(p["order"] for p in protocol["pairs"]), {"AB": 2, "BA": 2})
        self.assertEqual({p["repetition"] for p in protocol["pairs"]}, {1, 2, 3, 4})
        full.validate_protocol(protocol)

    def test_schema_growth_run_measures_the_configuration_tasks_against_the_last_ledger_candidate(self):
        from collections import Counter
        full = self.full_help_runner()
        regression = json.loads(full.PROTOCOL.with_name("regression-0.8-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-2026-10-23-study.json").read_text())
        # Tasks 3 and 8 read --schema config in every recorded session; the
        # 2026-10-23 desk review found the schemas 28% larger.
        self.assertEqual(protocol["task_ids"], [3, 8])
        self.assertEqual(protocol["maximum_sessions"], 16)
        self.assertFalse(protocol["previous_trials_included"])
        # The control is the last ledger candidate, the help-split desk review's.
        self.assertEqual(protocol["control"]["source_revision"], "3bec0a383c20ec0fe5902fb4e13b969b5f3be014")
        self.assertEqual(protocol["control"]["binary_sha256"],
                         "b1fc44c97df70c263b3930eb7628b90b872cb7bdb192a25fe428242627c13854")
        for key in ["model", "effort", "limits", "repetitions", "concurrent_pairs", "task_files"]:
            self.assertEqual(protocol[key], regression[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], regression["decision_rule"][key])
        for number in protocol["task_ids"]:
            selected = [p for p in protocol["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {1, 2, 3, 4})
        full.validate_protocol(protocol)

    def test_configuration_regression_adds_the_category_naming_task_against_the_last_ledger_candidate(self):
        from collections import Counter
        full = self.full_help_runner()
        regression = json.loads(full.PROTOCOL.with_name("regression-2026-10-23-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-study.json").read_text())
        # Tasks 3 and 8 configure projects with category names the prompt
        # gives; task 12 leaves the names to the agent, which the category
        # reference of the candidate addresses.
        self.assertEqual(protocol["task_ids"], [3, 8, 12])
        self.assertEqual(protocol["maximum_sessions"], 24)
        self.assertFalse(protocol["previous_trials_included"])
        # The control is the last ledger candidate.
        self.assertEqual(protocol["control"]["source_revision"], "bd820b7e2003172addba7169038de376634041ad")
        for key in ["model", "effort", "limits", "repetitions", "concurrent_pairs", "task_files"]:
            self.assertEqual(protocol[key], regression[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], regression["decision_rule"][key])
        for number in protocol["task_ids"]:
            selected = [p for p in protocol["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {1, 2, 3, 4})
        full.validate_protocol(protocol)

    def test_context_growth_sums_input_increases_after_the_first_request_and_counts_decreases(self):
        def request(tokens):
            return {"type": "event_msg", "payload": {"type": "token_count", "info": {"last_token_usage": {"input_tokens": tokens}}}}
        # The first request carries the shared instructions; a smaller request, as
        # after compaction, adds nothing but is counted.
        rows = [request(100), {"type": "event_msg", "payload": {"type": "token_count", "info": None}},
                request(150), request(140), request(200), {"type": "response_item", "payload": {}}]
        self.assertEqual(BENCH.context_growth(rows), {"context_growth": 110, "context_decreases": 1})
        self.assertEqual(BENCH.context_growth([]), {"context_growth": 0, "context_decreases": 0})

    def test_sessions_get_a_fresh_home_like_a_new_install_and_keep_refreshed_credentials(self):
        with tempfile.TemporaryDirectory() as folder:
            source = Path(folder) / "codex"
            (source / "plugins/cache").mkdir(parents=True)
            (source / "AGENTS.md").write_text("owner's global instructions")
            (source / "auth.json").write_text(json.dumps({"last_refresh": "2026-10-18T19:24:31Z", "tokens": "old"}))
            with BENCH.subject_home(source) as home:
                self.assertFalse(home.is_relative_to(Path(folder)))
                # Only the credentials: no instructions, plugins or personal skills.
                self.assertEqual(sorted(p.relative_to(home).as_posix() for p in home.rglob("*")), [".codex", ".codex/auth.json"])
                self.assertEqual((home / ".codex/auth.json").stat().st_mode & 0o777, 0o600)
                # Codex rotates the refresh token when it refreshes, so the copy's newer credentials must survive it.
                (home / ".codex/auth.json").write_text(json.dumps({"last_refresh": "2026-10-25T08:00:00Z", "tokens": "new"}))
            self.assertFalse(home.exists())
            self.assertEqual(json.loads((source / "auth.json").read_text())["tokens"], "new")
            self.assertEqual((source / "auth.json").stat().st_mode & 0o777, 0o600)
            with BENCH.subject_home(source) as home:
                (home / ".codex/auth.json").write_text(json.dumps({"last_refresh": "2026-10-20T00:00:00Z", "tokens": "stale"}))
            self.assertEqual(json.loads((source / "auth.json").read_text())["tokens"], "new")

    def test_context_audit_flags_injected_instructions_and_fingerprints_what_every_session_shares(self):
        def message(role, text):
            return {"type": "response_item", "payload": {"role": role, "content": [{"type": "input_text", "text": text}]}}
        def session(home, prompt):
            skills = f"<skills_instructions>\n- `r0` = `{home}/.codex/skills/.system`\n- imagegen: Images (file: r0/imagegen/SKILL.md)"
            return [{"type": "session_meta", "payload": {"base_instructions": {"text": "You are Codex"}}},
                    {"type": "turn_context", "payload": {"model": "gpt-6.1-sol", "effort": "xhigh"}},
                    message("developer", skills), message("developer", f"<permissions instructions>\nwrite {home}"),
                    message("user", f"<environment_context>{home}"), message("user", prompt),
                    {"type": "response_item", "payload": {"type": "function_call"}}]
        first, second = BENCH.context_audit(session("/tmp/a", "Task one")), BENCH.context_audit(session("/tmp/b", "Task two"))
        self.assertEqual({k: first[k] for k in ["repository_context", "global_instructions", "configurations", "skills"]},
                         {"repository_context": False, "global_instructions": False,
                          "configurations": [{"model": "gpt-6.1-sol", "effort": "xhigh"}], "skills": ["imagegen"]})
        # Paths and the task differ per session; the instructions and skills do not.
        self.assertEqual(first["fingerprint"], second["fingerprint"])
        changed = session("/tmp/a", "Task one")
        changed[0]["payload"]["base_instructions"]["text"] = "You are Codex, revised"
        self.assertNotEqual(BENCH.context_audit(changed)["fingerprint"], first["fingerprint"])
        audit = BENCH.context_audit([*session("/tmp/a", "x"), message("user", "# AGENTS.md instructions\n\n<INSTRUCTIONS>")])
        self.assertTrue(audit["global_instructions"])
        self.assertTrue(BENCH.context_audit([message("user", "# Repository Guidelines")])["repository_context"])

    def test_a_campaign_records_the_context_fingerprints_of_its_sessions(self):
        with tempfile.TemporaryDirectory() as folder:
            destination = Path(folder)
            for run, fingerprint in [("task1-rep1/evidence/run-01", "a"), ("task1-rep1/evidence/run-02", "a"),
                                     ("task3-rep1/evidence/run-01", "b")]:
                (destination / run).mkdir(parents=True)
                (destination / run / "context-audit.json").write_text(json.dumps({"fingerprint": fingerprint}))
            self.assertEqual(BENCH.context_fingerprints(destination), {"a": 2, "b": 1})

    def test_sections_regression_adds_an_unrelated_reader_and_freezes_one_confirmation_sample(self):
        from collections import Counter
        full = self.full_help_runner()
        regression = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-study.json").read_text())
        protocol = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-sections-study.json").read_text())
        # Task 1 guards root-help reading for an unrelated task (docs/help-design.md);
        # tasks 3, 8 and 12 read configuration references.
        self.assertEqual(protocol["task_ids"], [1, 3, 8, 12])
        self.assertEqual(protocol["maximum_sessions"], 32)
        self.assertFalse(protocol["previous_trials_included"])
        self.assertEqual(protocol["control"], regression["control"])
        for key in ["model", "effort", "limits", "repetitions", "concurrent_pairs", "task_files"]:
            self.assertEqual(protocol[key], regression[key])
        for key in ["correctness", "per_task_medians", "overall_all_run_totals", "secondary", "zero_comparator"]:
            self.assertEqual(protocol["decision_rule"][key], regression["decision_rule"][key])
        self.assertIn("context_growth", protocol["decision_rule"]["diagnostics"])
        confirmation = protocol["confirmation"]
        self.assertTrue(confirmation["pooled"])
        self.assertEqual(confirmation["maximum_sessions"], 32)
        for number in protocol["task_ids"]:
            selected = [p for p in confirmation["pairs"] if p["task"] == number]
            self.assertEqual(Counter(p["order"] for p in selected), {"AB": 2, "BA": 2})
            self.assertEqual({p["repetition"] for p in selected}, {5, 6, 7, 8})
        full.validate_protocol(protocol)
        unbalanced = json.loads(json.dumps(protocol))
        unbalanced["confirmation"]["pairs"][0]["order"] = "BA" if unbalanced["confirmation"]["pairs"][0]["order"] == "AB" else "AB"
        with self.assertRaises(ValueError):
            full.validate_protocol(unbalanced)

    def test_a_confirmation_sample_runs_once_for_named_tasks_after_the_primary_sample(self):
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.with_name("regression-2026-10-24-sections-study.json").read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            destination = root / "campaign"
            with patch.object(full.BENCH, "REPO", BENCH.REPO), patch.object(full.BENCH, "run_agent") as run:
                full.campaign(destination, binaries, ["control-revision", "candidate-revision"], protocol_path, prepare_only=True)
                self.assertEqual(len(list(destination.glob("task*-rep*/manifest.json"))), 32)
                with self.assertRaisesRegex(ValueError, "primary sample"):
                    full.run_confirmation(destination, ACK, [12])
                full.run_frozen(destination, ACK)
                self.assertEqual(run.call_count, 32)
                self.assertFalse(any(call.args[0].name.endswith(("rep5", "rep6", "rep7", "rep8")) for call in run.call_args_list))
                with self.assertRaisesRegex(ValueError, "not in this protocol"):
                    full.run_confirmation(destination, ACK, [2])
                full.run_confirmation(destination, ACK, [12])
                confirmed = [call.args[0].name for call in run.call_args_list[32:]]
                self.assertEqual(sorted(set(confirmed)), [f"task12-rep{n}" for n in [5, 6, 7, 8]])
                self.assertEqual(run.call_count, 40)
                with self.assertRaisesRegex(ValueError, "already ran"):
                    full.run_confirmation(destination, ACK, [3])
                self.assertEqual(run.call_count, 40)

    def test_full_help_grader_delegates_the_category_naming_task_to_the_artifact_checks(self):
        spec = importlib.util.spec_from_file_location("full_help_score", Path(__file__).with_name("score_full_help.py"))
        grader = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(grader)
        with tempfile.TemporaryDirectory() as tmp:
            pair = Path(tmp)
            task = json.loads(BENCH.TASK.with_name("task12.json").read_text())
            (pair / "manifest.json").write_text(json.dumps({"task": task}))
            evidence = pair / "evidence/run-01"
            evidence.mkdir(parents=True)
            (evidence / "summary.json").write_text(json.dumps({"cli_invocations": 5}))
            with patch.object(grader.LEGACY, "grade", return_value={"task": 12}) as legacy:
                result = grader.grade(pair, "run-01")
            legacy.assert_called_once_with(pair.resolve(), "run-01")
            self.assertEqual(result, {"task": 12, "explorer_cli_invocations": 5})

    def test_full_help_grader_checks_the_configuration_layout_task_with_its_legacy_oracle(self):
        spec = importlib.util.spec_from_file_location("full_help_score", Path(__file__).with_name("score_full_help.py"))
        grader = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(grader)
        with tempfile.TemporaryDirectory() as tmp:
            pair = Path(tmp)
            task = json.loads(BENCH.TASK.with_name("task8.json").read_text())
            (pair / "manifest.json").write_text(json.dumps({"task": task}))
            evidence = pair / "evidence/run-01"
            evidence.mkdir(parents=True)
            (evidence / "summary.json").write_text(json.dumps({"cli_invocations": 5}))
            with patch.object(grader.LEGACY, "grade", return_value={"task": 8}) as legacy:
                result = grader.grade(pair, "run-01")
            legacy.assert_called_once_with(pair.resolve(), "run-01")
            self.assertEqual(result, {"task": 8, "explorer_cli_invocations": 5})

    def test_runner_rejects_unknown_protocols_or_changed_fixed_task_and_session_counts(self):
        full = self.full_help_runner()
        for path in [full.PROTOCOL, full.PROTOCOL.with_name("compare-help-routing-study.json"),
                     full.PROTOCOL.with_name("regression-0.8-study.json"),
                     full.PROTOCOL.with_name("regression-0.8-correction-study.json"),
                     full.PROTOCOL.with_name("regression-2026-10-23-study.json")]:
            protocol = json.loads(path.read_text())
            full.validate_protocol(protocol)
            for key, value in [("revision", "unrecognized-study"), ("task_ids", [1, 4]), ("maximum_sessions", 48)]:
                with self.subTest(protocol=path.name, changed=key), self.assertRaises(ValueError):
                    full.validate_protocol({**protocol, key: value})

    def test_comparison_correction_freezes_only_four_tasks_and_stops_after_32_sessions(self):
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.with_name("compare-help-routing-study.json").read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            destination = root / "campaign"
            with patch.object(full.BENCH, "REPO", BENCH.REPO), patch.object(full.BENCH, "run_agent") as run:
                full.campaign(destination, binaries, ["control-revision", "candidate-revision"], protocol_path, prepare_only=True)
                run.assert_not_called()
                self.assertEqual(len(list(destination.glob("task*-rep*/manifest.json"))), 16)
                self.assertEqual({p.name for p in (destination / "tasks").iterdir()},
                                 {"task1.json", "task4.json", "task5.json", "task7-full-help.json"})
                plan = json.loads((destination / "campaign.json").read_text())
                self.assertEqual(set(plan["task_hashes"]), {"1", "4", "5", "7"})
                frozen_protocol = destination / "protocol.json"
                frozen_text = frozen_protocol.read_text()
                frozen_protocol.write_text(frozen_text + " ")
                with self.assertRaisesRegex(ValueError, "Frozen input changed"):
                    full.run_frozen(destination, ACK)
                run.assert_not_called()
                self.assertFalse((destination / "started.json").exists())
                frozen_protocol.write_text(frozen_text)
                full.run_frozen(destination, ACK)
                self.assertEqual(run.call_count, 32)
                self.assertEqual(json.loads((destination / "started.json").read_text())["sessions"], 32)
                for item in protocol["pairs"]:
                    target = destination / f"task{item['task']}-rep{item['repetition']}"
                    actual = [call.args[1] for call in run.call_args_list if call.args[0] == target]
                    self.assertEqual(actual, [{"A": "run-02", "B": "run-01"}[arm] for arm in item["order"]])
                with self.assertRaisesRegex(ValueError, "already started"):
                    full.run_frozen(destination, ACK)
                self.assertEqual(run.call_count, 32)

    def test_full_help_freezes_inputs_before_running_and_uses_only_frozen_binaries(self):
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixtures = root / "fixtures"
            shutil.copytree(BENCH.TASK.parent, fixtures)
            for number in [10, 11]:
                if not (fixtures / f"task{number}.json").exists():
                    shutil.copyfile(fixtures / "task1.json", fixtures / f"task{number}.json")
            binaries = [root / arm for arm in "AB"]
            for arm, binary in zip("AB", binaries):
                binary.write_text(arm)
            protocol = json.loads(full.PROTOCOL.read_text())
            protocol["control"] = {"source_revision": "control-revision", "binary_sha256": BENCH.sha(binaries[0])}
            protocol_path = root / "protocol.json"
            protocol_path.write_text(json.dumps(protocol))
            destination = root / "campaign"
            with patch.object(full.BENCH, "REPO", BENCH.REPO), patch.object(full.BENCH, "TASK", fixtures / "task1.json"), patch.object(full.BENCH, "run_agent") as run:
                full.campaign(destination, binaries, ["control-revision", "candidate-revision"], protocol_path, prepare_only=True)
                run.assert_not_called()
                plan = json.loads((destination / "campaign.json").read_text())
                self.assertEqual(plan["arms"]["A"]["source_revision"], "control-revision")
                self.assertEqual(plan["arms"]["B"]["source_revision"], "candidate-revision")
                grader = BENCH.REPO / "scripts/score_full_help.py"
                self.assertEqual(plan["full_help_grader_sha256"], BENCH.sha(grader))
                self.assertEqual((destination / "scripts/score_full_help.py").read_bytes(), grader.read_bytes())
                self.assertEqual(len(list(destination.glob("task*-rep*/manifest.json"))), 40)
                frozen = destination / "tasks/task7-full-help.json"
                original = frozen.read_text()
                frozen.write_text("changed frozen task")
                with self.assertRaisesRegex(ValueError, "Frozen input changed"):
                    full.run_frozen(destination, ACK)
                run.assert_not_called()
                self.assertFalse((destination / "started.json").exists())
                frozen.write_text(original)
                original_grader = grader.read_text()
                grader.write_text(original_grader + "\n")
                with self.assertRaisesRegex(ValueError, "Study code changed after freezing: score_full_help.py"):
                    full.run_frozen(destination, ACK)
                run.assert_not_called()
                self.assertFalse((destination / "started.json").exists())
                grader.write_text(original_grader)
                manifest = destination / "campaign.json"
                original_manifest = manifest.read_text()
                manifest.write_text(original_manifest + " ")
                with self.assertRaisesRegex(ValueError, "manifest changed"):
                    full.run_frozen(destination, ACK)
                manifest.write_text(original_manifest)
                binaries[0].write_text("changed after freeze")
                protocol_path.write_text("changed after freeze")
                lock = threading.Lock()
                active = set()
                peak = 0

                def observed_run(target, run, model, effort):
                    nonlocal peak
                    with lock:
                        self.assertNotIn(target, active, "Arms of a pair must run sequentially")
                        active.add(target)
                        peak = max(peak, len(active))
                    time.sleep(0.005)
                    with lock:
                        active.remove(target)

                run.side_effect = observed_run
                full.run_frozen(destination, ACK)
                self.assertEqual(run.call_count, 80)
                self.assertLessEqual(peak, 3)
                for item in protocol["pairs"]:
                    target = destination / f"task{item['task']}-rep{item['repetition']}"
                    actual = [call.args[1] for call in run.call_args_list if call.args[0] == target]
                    self.assertEqual(actual, [{"A": "run-02", "B": "run-01"}[a] for a in item["order"]])
                    self.assertEqual((target / "A/target/debug/locmetry").read_text(), "A")
                self.assertTrue((destination / "started.json").exists())
                self.assertEqual(json.loads((destination / "started.json").read_text())["sessions"], 80)
                with self.assertRaisesRegex(ValueError, "already started"):
                    full.run_frozen(destination, ACK)

    def test_full_help_rejects_wrong_control_before_creating_output(self):
        full = self.full_help_runner()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("binary")
            with self.assertRaisesRegex(ValueError, "control"):
                full.campaign(root / "campaign", [binary] * 2, ["wrong", "candidate"], prepare_only=True)
            self.assertFalse((root / "campaign").exists())

    def test_campaigns_run_only_where_the_explorer_sandbox_lets_the_launcher_log_calls(self):
        # Codex's workspace-write sandbox allows writes only below the run, /tmp and
        # $TMPDIR, and the launcher logs beside the run; the first 0.8.0 regression
        # campaign, frozen inside the repository, recorded no CLI call (2026-10-22).
        full = self.full_help_runner()
        outside = Path.home() / "locmetry-campaign-never-created"
        with tempfile.TemporaryDirectory() as tmp:
            binary = Path(tmp) / "binary"
            binary.write_text("binary")
            with self.assertRaisesRegex(ValueError, "sandbox"):
                full.campaign(outside, [binary] * 2, ["control", "candidate"], prepare_only=True)
        self.assertFalse(outside.exists())
        with tempfile.TemporaryDirectory() as tmp:
            destination = Path(tmp) / "campaign"
            destination.mkdir()
            with patch.object(full, "EXPLORER_WRITABLE_ROOTS", ()), \
                    self.assertRaisesRegex(ValueError, "sandbox"):
                full.run_frozen(destination, ACK)
            self.assertFalse((destination / "started.json").exists())

    def test_an_unfinished_last_telemetry_record_does_not_hide_earlier_events(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "events.jsonl"
            path.write_text('{"event": 1}\n{"event":')
            self.assertEqual(BENCH.records(path), [{"event": 1}])

    def test_corrupt_completed_telemetry_cannot_silently_lower_call_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "events.jsonl"
            path.write_text('{"event": 1}\nbroken\n{"event": 3}\n')
            with self.assertRaises(json.JSONDecodeError):
                BENCH.records(path)

    def test_task_seed_does_not_substitute_the_report_understanding_fixture(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            BENCH.seed(root, {"files": {"projects/partial/empty.py": ""}})
            self.assertTrue((root / "projects/partial/empty.py").exists())
            self.assertFalse((root / "projects/web").exists())

    def test_help_recipe_treatment_retains_guides_and_other_product_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            pair = Path(tmp) / "pair"
            BENCH.prepare(pair, treatment="help-recipe")
            a = (pair / "A/src/main.rs").read_text()
            b = (pair / "B/src/main.rs").read_text()
            self.assertIn("guide: Option<Guide>", b)
            self.assertIn("Classify maintained scripts", b)
            self.assertNotIn("Classify maintained scripts", a)
            for path in (pair / "A/src").glob("*.rs"):
                if path.name != "main.rs":
                    self.assertEqual(path.read_bytes(), (pair / "B/src" / path.name).read_bytes())
            manifest = json.loads((pair / "manifest.json").read_text())
            self.assertEqual(manifest["treatment"], "help-recipe")

    def test_routing_trial_isolates_wording_from_the_recipe_and_maps_three_binaries(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binaries = []
            for arm in ["A", "B", "C"]:
                binary = root / arm
                binary.write_text(arm)
                binaries.append(binary)
            pair = root / "triad"
            BENCH.prepare(pair, binaries=binaries, treatment="help-routing")
            manifest = json.loads((pair / "manifest.json").read_text())
            self.assertEqual(manifest["order"], {"run-01": "B", "run-02": "A", "run-03": "C"})
            routing = (BENCH.TASK.parent / "help-routing.txt").read_text().replace('\\', '\\\\').replace('"', '\\"')
            b = (pair / "B/src/main.rs").read_text()
            c = (pair / "C/src/main.rs").read_text()
            self.assertEqual(c.count(routing), 1)
            self.assertEqual(c.replace(routing, ""), b)
            for arm in ["A", "B", "C"]:
                self.assertEqual((pair / arm / "target/debug/locmetry").read_text(), arm)
                for path in (pair / "A/src").rglob("*.rs"):
                    if path.name != "main.rs":
                        self.assertEqual(path.read_bytes(), (pair / arm / path.relative_to(pair / "A")).read_bytes())
            self.assertTrue((pair / "run-03/projects/web").is_dir())

    def test_routing_schedule_uses_every_permutation_and_executes_frozen_arms(self):
        from collections import Counter
        spec = importlib.util.spec_from_file_location("routing", Path(__file__).with_name("agent-routing-study.py"))
        routing = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(routing)
        protocol = json.loads(routing.PROTOCOL.read_text())
        self.assertEqual(Counter(t["order"] for t in protocol["triads"]),
                         {"ABC": 2, "ACB": 2, "BAC": 2, "BCA": 2, "CAB": 2, "CBA": 2})
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("test binary")
            with patch.object(routing.BENCH, "prepare"), patch.object(routing.BENCH, "run_agent") as run:
                routing.campaign(root / "campaign", [binary] * 3)
            self.assertEqual(run.call_count, 36)
            for item in protocol["triads"]:
                target = root / "campaign" / f"task{item['task']}-rep{item['repetition']}"
                actual = [c.args[1] for c in run.call_args_list if c.args[0] == target]
                self.assertEqual(actual, [{"A": "run-02", "B": "run-01", "C": "run-03"}[a] for a in item["order"]])

    def test_routing_replication_uses_its_own_frozen_protocol(self):
        spec = importlib.util.spec_from_file_location("routing", Path(__file__).with_name("agent-routing-study.py"))
        routing = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(routing)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("test binary")
            protocol = json.loads(routing.PROTOCOL.read_text())
            protocol.update(revision="replication-test", task_ids=[8], repetitions=1,
                            triads=[{"task": 8, "repetition": 1, "order": "CBA"}])
            definition = root / "protocol.json"
            definition.write_text(json.dumps(protocol))
            with patch.object(routing.BENCH, "prepare"), patch.object(routing.BENCH, "run_agent") as run:
                routing.campaign(root / "campaign", [binary] * 3, definition)
            self.assertEqual([c.args[1] for c in run.call_args_list], ["run-03", "run-01", "run-02"])
            plan = json.loads((root / "campaign/campaign.json").read_text())
            self.assertEqual(plan["revision"], "replication-test")
            self.assertEqual(plan["protocol_sha256"], routing.BENCH.sha(definition))
            self.assertEqual(set(plan["task_hashes"]), {"8"})

    def test_campaign_honors_selected_tasks_repetitions_and_treatment(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("test binary")
            with patch.object(BENCH, "run_agent") as run:
                BENCH.campaign(root / "campaign", [binary, binary], "test", "test",
                               treatment="help-recipe", task_ids=[1, 3], repetitions=2)
            self.assertEqual(run.call_count, 8)
            plan = json.loads((root / "campaign/campaign.json").read_text())
            self.assertEqual(plan["treatment"], "help-recipe")
            self.assertEqual({p["task"] for p in plan["pairs"]}, {1, 3})
            self.assertEqual(sum(p["first"] == "A" for p in plan["pairs"]), 2)
            manifests = list((root / "campaign").glob("task*-rep*/manifest.json"))
            self.assertEqual(len(manifests), 4)
            self.assertTrue(all(json.loads(p.read_text())["treatment"] == "help-recipe" for p in manifests))

    def test_independent_config_case_rejects_wrong_saved_decisions(self):
        self.check_configuration_case(8)

    def test_simple_config_case_rejects_wrong_saved_decisions(self):
        self.check_configuration_case(9)

    def test_category_naming_case_rejects_merged_or_default_categories(self):
        # Task 12 grades purposes, counts and that the requested kinds stay
        # apart; the names themselves are a diagnostic, not an oracle.
        task = json.loads(BENCH.TASK.with_name("task12.json").read_text())
        names = task["oracle"]["reference_categories"]
        for file, category in [(None, None), ("tests/cli.rs", "unit"), ("ci/run-tests.sh", "implementation"),
                               ("tests/common/mod.rs", "test")]:
            categories = dict(names, **({file: category} if file else {}))
            outcome = self.check_configuration_case(12, categories, expect_pass=file is None)
            self.assertEqual(outcome, "artifact_checks_pass" if file is None else "needs_review", file)

    def check_configuration_case(self, number, categories=None, expect_pass=None):
        task = json.loads(BENCH.TASK.with_name(f"task{number}.json").read_text())
        oracle = task["oracle"]
        report = {"summary": {"source": oracle["source"], "all_counted": oracle["all_counted"],
                              "source_purpose": {k: {"code": v} for k, v in oracle["purpose_code"].items()}},
                  "files": [{"path": p, **v, **({"category": categories[p]} if categories and p in categories else {})}
                            for p, v in oracle["decisions"].items()],
                  "review": [{"path": p} for p in oracle["review"]]}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence/run-02"
            evidence.mkdir(parents=True)
            (root / "manifest.json").write_text(json.dumps({"task": task}))
            (evidence / "summary.json").write_text(json.dumps({"condition": "A", "returncode": 0, "stopped": None, "fixture_intact": True, "cli_invocations": 0}))
            (evidence / "context-audit.json").write_text(json.dumps({"repository_context": False, "configurations": [{"model": "test"}]}))
            (evidence / "calls.jsonl").write_text("")
            for name in oracle["artifacts"]:
                target = root / "run-02" / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(json.dumps(report) if name.endswith(".json") else "version = 1")
            actual = SCORING.subprocess.CompletedProcess([], 0, json.dumps(report), "")
            with patch.object(SCORING.subprocess, "run", return_value=actual):
                if expect_pass is not None:
                    return SCORING.grade(root, "run-02")["outcome"]
                self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "artifact_checks_pass")
                report["files"][0]["category"] = "incorrect"
                for bad in [report, {}, []]:
                    (root / "run-02/outputs/configured.json").write_text(json.dumps(bad))
                    self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "needs_review")
                task["revision"] = "task10-unsupported-v1"
                (root / "manifest.json").write_text(json.dumps({"task": task}))
                self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "needs_review")

    def test_process_success_does_not_mask_incorrect_delivered_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence/run-02"
            evidence.mkdir(parents=True)
            task = json.loads(BENCH.TASK.read_text())
            (root / "manifest.json").write_text(json.dumps({"task": task}))
            (evidence / "summary.json").write_text(json.dumps({"condition": "A", "returncode": 0, "stopped": None, "fixture_intact": True, "cli_invocations": 1}))
            (evidence / "context-audit.json").write_text(json.dumps({"repository_context": False, "configurations": [{"model": "test"}]}))
            (evidence / "calls.jsonl").write_text(json.dumps({"id": "report", "args": ["--format", "json"]}) + "\n")
            summary = {key: dict(task["oracle"][key]) for key in ["source", "all_counted"]}
            (evidence / "report.stdout").write_text(json.dumps({"summary": summary}))
            self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "artifact_checks_pass")
            summary["source"]["code"] = 999
            (evidence / "report.stdout").write_text(json.dumps({"summary": summary}))
            result = SCORING.grade(root, "run-02")
            self.assertEqual(result["outcome"], "needs_review")
            self.assertFalse(result["checks"]["source:code"])


if __name__ == "__main__":
    unittest.main()
