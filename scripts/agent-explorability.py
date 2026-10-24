"""Prepare isolated CLI discovery experiments; never edit product sources."""

import argparse
import collections
from concurrent.futures import ThreadPoolExecutor
import contextlib
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile
import threading
import time


REPO = Path(__file__).resolve().parent.parent
TASK = REPO / "tests/fixtures/agent-explorability/task1.json"
CODEX_HOME = Path.home() / ".codex"
# One parent for every subject home, so that a run can find what an interrupted run left.
SUBJECT_HOMES = Path(tempfile.gettempdir()) / "locmetry-subject-homes"
CREDENTIALS = threading.Lock()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def seed(root, task=None):
    task = task or json.loads(TASK.read_text())
    for name, contents in task["files"].items():
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)


def without_guides(source):
    """Remove guide access and references, retaining other wording and semantics."""
    changes = {
        "src/main.rs": [
            ("mod guides;\n", ""),
            ("use guides::Guide;\n", ""),
            (" For optional project configuration, use locmetry --guide configure.", ""),
            ("\n\nFor workflows and saved-report recipes, use locmetry --guide TOPIC with a topic listed under --guide.", ""),
            ("    /// Read a task guide. Use --help for syntax, --guide for workflows and\n    /// caveats, and --schema for machine-readable contracts. Standalone query.\n    #[arg(long, value_enum)]\n    guide: Option<Guide>,\n", ""),
            (" For optional configuration, use locmetry --guide configure; for results, use locmetry --guide reports.", ""),
            ('    #[command(after_help = "For baselines and interpretation, use locmetry --guide compare.")]\n', ""),
            ('    #[command(\n        after_help = "For selection and classification diagnosis, use locmetry --guide selection."\n    )]\n', ""),
            ('    if let Some(guide) = cli.guide {\n        let (path, text) = guide.document();\n        let mut stdout = io::stdout().lock();\n        writeln!(\n            stdout,\n            "Source: {path}; locmetry {}\\n",\n            env!("CARGO_PKG_VERSION")\n        )?;\n        stdout.write_all(text.as_bytes())?;\n        return Ok(Outcome::Complete);\n    }\n', ""),
            ('for query in ["schema", "guide"]', 'for query in ["schema"]'),
        ],
        "src/output.rs": [
            ("To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.", "--files --output - shows each file's rule."),
        ],
        "src/config.rs": [
            ("start with `locmetry --guide configure` and measure", "measure"),
        ],
    }
    patch = []
    for name, substitutions in changes.items():
        path = source / name
        original = text = path.read_text()
        for before, after in substitutions:
            if text.count(before) != 1:
                raise ValueError(f"Treatment no longer matches exactly once: {name}: {before!r}")
            text = text.replace(before, after)
        path.write_text(text)
        patch.extend(difflib.unified_diff(original.splitlines(True), text.splitlines(True), f"a/{name}", f"b/{name}"))
    (source.parent / "treatment.patch").write_text("".join(patch))


def with_help_recipe(source, routing=False):
    path = source / "src/main.rs"
    original = path.read_text()
    marker = "For workflows and saved-report recipes, use locmetry --guide TOPIC"
    if original.count(marker) != 1:
        raise ValueError("Help recipe insertion point must match exactly once")
    recipe = (TASK.parent / "help-recipe.txt").read_text()
    if routing:
        recipe = (TASK.parent / "help-routing.txt").read_text() + recipe
    escaped = recipe.replace('\\', '\\\\').replace('"', '\\"')
    changed = original.replace(marker, escaped + marker)
    path.write_text(changed)
    patch = difflib.unified_diff(original.splitlines(True), changed.splitlines(True), "a/src/main.rs", "b/src/main.rs")
    (source.parent / "treatment.patch").write_text("".join(patch))


def share_binary(binary, target):
    """Hard-link a frozen binary into a pair, copying only where no link can be made.

    Per-pair copies made the 0.8.0 regression run's evidence 905 MB (2026-10-22).
    Sharing one file is safe because audits hash each pair's binary after the run.
    """
    try:
        os.link(binary, target)
    except OSError:
        shutil.copy2(binary, target)


def prepare(destination, task_path=TASK, binaries=None, treatment="without-guides", *, arm_provenance=None):
    supplied = treatment == "supplied-binaries"
    if treatment not in ["without-guides", "help-recipe", "help-routing", "supplied-binaries"]:
        raise ValueError(f"Unknown treatment: {treatment}")
    help_sources = [REPO / "src/main.rs", REPO / "src/app/cli.rs", REPO / "src/app/cli/long_help.rs"]
    if not supplied and any(path.is_file() and "Classify maintained scripts in service/.locmetryrc" in path.read_text() for path in help_sources):
        raise ValueError("Historical treatments require pre-adoption source; use the study's recorded source revision in a separate checkout, or define a new study for the current interface")
    conditions = ["A", "B", "C"] if treatment == "help-routing" else ["A", "B"]
    if supplied and (not binaries or len(binaries) != 2):
        raise ValueError("Supplied-binaries mode requires exactly two binaries")
    if binaries and len(binaries) != len(conditions):
        raise ValueError("One binary is required per treatment condition")
    if supplied:
        for binary in binaries:
            if not binary.is_file():
                raise FileNotFoundError(binary)
        if arm_provenance is not None and set(arm_provenance) != set(conditions):
            raise ValueError("Source provenance must identify both supplied arms")
    destination.mkdir(parents=True, exist_ok=False)
    files = [REPO / "Cargo.toml", REPO / "Cargo.lock"]
    files += list((REPO / "src").rglob("*.rs"))
    files += list((REPO / "docs/guides").glob("*.md"))
    if not supplied:
        for condition in conditions:
            for path in files:
                target = destination / condition / path.relative_to(REPO)
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(path, target)
    if treatment == "without-guides":
        without_guides(destination / "B")
    elif treatment == "help-recipe":
        with_help_recipe(destination / "B")
    elif treatment == "help-routing":
        with_help_recipe(destination / "B")
        (destination / "treatment.patch").rename(destination / "B.patch")
        with_help_recipe(destination / "C", routing=True)
        (destination / "treatment.patch").rename(destination / "C.patch")
    if binaries:
        for condition, binary in zip(conditions, binaries):
            target = destination / condition / "target/debug/locmetry"
            target.parent.mkdir(parents=True)
            share_binary(binary, target)
    task = json.loads(task_path.read_text())
    manifest = {
        "revision": task["revision"],
        "treatment": treatment,
        "task_sha256": sha(task_path),
        "task": task,
        "harness_sha256": sha(Path(__file__)),
        "limits": task["limits"],
        "order": {"run-01": "B", "run-02": "A"},
        "environment": {"COLUMNS": "100", "NO_COLOR": "1", "LC_ALL": "C", "stdout_tty": False},
    }
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
    source_hashes = {str(p.relative_to(REPO)): sha(p) for p in files}
    if supplied:
        manifest["repository_snapshot"] = {"revision": revision, "source_hashes": source_hashes}
        manifest["arms"] = {
            arm: {**(arm_provenance or {}).get(arm, {}), "binary_sha256": sha(binary)}
            for arm, binary in zip(conditions, binaries)
        }
    else:
        manifest.update(source_revision=revision, source_hashes=source_hashes)
    if treatment == "help-routing":
        manifest["order"]["run-03"] = "C"
    for run, condition in manifest["order"].items():
        root = destination / run
        seed(root, task)
        (root / "bin").mkdir()
        logs = destination / "evidence" / run
        logs.mkdir(parents=True)
        config = destination / "user-config" / run
        config.mkdir(parents=True)
        if "user_defaults" in task:
            defaults = config / "locmetry/config.toml"
            defaults.parent.mkdir()
            defaults.write_text(task["user_defaults"])
        executable = destination / condition / "target/debug/locmetry"
        wrapper = '''#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys, time, uuid
logs = pathlib.Path(__LOGS__)
env = dict(os.environ, XDG_CONFIG_HOME=__CONFIG__, COLUMNS="100", NO_COLOR="1", LC_ALL="C")
started = time.time()
result = subprocess.run([sys.argv[0], *sys.argv[1:]], executable=__EXECUTABLE__, capture_output=True, env=env)
identity = uuid.uuid4().hex
(logs / (identity + ".stdout")).write_bytes(result.stdout)
(logs / (identity + ".stderr")).write_bytes(result.stderr)
record = dict(id=identity, started=started, elapsed=time.time()-started, args=sys.argv[1:], cwd=os.getcwd(), exit=result.returncode, stdout_bytes=len(result.stdout), stderr_bytes=len(result.stderr))
with (logs / "calls.jsonl").open("a") as handle:
    handle.write(json.dumps(record) + "\\n")
sys.stdout.buffer.write(result.stdout)
sys.stderr.buffer.write(result.stderr)
sys.exit(result.returncode)
'''
        wrapper = wrapper.replace("__LOGS__", repr(str(logs))).replace("__CONFIG__", repr(str(config))).replace("__EXECUTABLE__", repr(str(executable)))
        launcher = root / "bin/locmetry"
        launcher.write_text(wrapper)
        launcher.chmod(0o755)
        (logs / "prompt.txt").write_text(task["prompt"])
    (destination / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(destination)


def normalize(value):
    if isinstance(value, dict):
        return {k: normalize(v) for k, v in value.items() if k != "measured_at_unix_seconds"}
    if isinstance(value, list):
        return [normalize(v) for v in value]
    return value


def verify(first, second):
    with tempfile.TemporaryDirectory(prefix="locmetry-task1-oracle-") as temp:
        root = Path(temp)
        seed(root)
        env = dict(os.environ, XDG_CONFIG_HOME=str(root / "user-config"), COLUMNS="100", NO_COLOR="1", LC_ALL="C")

        def run(binary, args):
            return subprocess.run([str(binary), *args], cwd=root, env=env, capture_output=True, text=True)

        for args in [["--help"], ["-h"], ["help"], *[[name, "--help"] for name in ["workspace", "compare", "explain"]]]:
            result = run(second, args)
            assert result.returncode == 0, result.stderr
            assert "--guide" not in result.stdout, "Guide still advertised in B help"
        for topic in ["configure", "selection", "reports", "compare"]:
            assert run(first, ["--guide", topic]).returncode == 0
            result = run(second, ["--guide", topic])
            assert result.returncode != 0 and not result.stdout, "Guide still available in B"
        for schema in ["report", "comparison", "workspace", "explanation", "user-config", "config"]:
            a = run(first, ["--schema", schema])
            b = run(second, ["--schema", schema])
            assert a.returncode == b.returncode == 0
            expected = a.stdout.replace("start with `locmetry --guide configure` and measure", "measure")
            assert expected == b.stdout and "--guide" not in b.stdout, schema
        for fmt in ["json", "headline", "table", "markdown"]:
            args = ["--root", "projects/web", "--format", fmt]
            if fmt == "json":
                args += ["--files"]
            if fmt == "table":
                args += ["--verbose", "--categories", "--no-bars"]
            a, b = run(first, args), run(second, args)
            assert a.returncode == b.returncode == 0, (a.stderr, b.stderr)
            if fmt in ["json", "headline"]:
                left, right = json.loads(a.stdout), json.loads(b.stdout)
                assert normalize(left) == normalize(right), fmt
                oracle = json.loads(TASK.read_text())["oracle"]
                for name in ["source", "all_counted"]:
                    for key, expected in oracle[name].items():
                        assert left["summary"][name][key] == expected, (name, key)
                for purpose, expected in oracle["purpose_code"].items():
                    assert left["summary"]["source_purpose"][purpose]["code"] == expected
                if fmt == "json":
                    assert left["metadata"]["coverage"]["unrecognized"] == oracle["unrecognized"]
                    assert left["metadata"]["backend_inaccurate"] is False
            else:
                def human(text):
                    return "\n".join(line for line in text.splitlines() if "Measured at:" not in line and "Measured:" not in line)
                expected = a.stdout.replace("To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.", "--files --output - shows each file's rule.")
                assert human(expected) == human(b.stdout), fmt
                assert "--guide" not in b.stdout
        assert run(first, ["--version"]).stdout == run(second, ["--version"]).stdout
    print("PASS: guide removal, retained schema contracts, fixture oracle, JSON/headline and human-output parity")


def records(path):
    if not path.exists():
        return []
    result = []
    lines = path.read_text().splitlines(keepends=True)
    for index, line in enumerate(lines):
        try:
            result.append(json.loads(line))
        except json.JSONDecodeError:
            # Session writers can expose an unfinished last line; completed corrupt
            # records must fail because ignoring them would undercount tool calls.
            if index == len(lines) - 1 and not line.endswith("\n"):
                continue
            raise
    return result


def token_counts(rows):
    """The usage records Codex logs after each model request."""
    return [r["payload"]["info"] for r in rows
            if r.get("type") == "event_msg" and r["payload"].get("type") == "token_count" and r["payload"].get("info")]


def context_growth(rows):
    """The context a session built, independent of provider cache hits.

    Summed increases of per-request input after the first request, which holds
    the instructions every session shares; a request smaller than its
    predecessor, as after compaction, adds nothing and is counted instead. The
    first diagnostic of 2026-10-24 subtracted the first request's cached input,
    which cross-session cache hits make vary (docs/research/agent-regression-run-2026-10-24.md).
    """
    inputs = [info["last_token_usage"]["input_tokens"] for info in token_counts(rows)]
    steps = [after - before for before, after in zip(inputs, inputs[1:])]
    return {"context_growth": sum(step for step in steps if step > 0), "context_decreases": sum(step < 0 for step in steps)}


@contextlib.contextmanager
def subject_home(source=CODEX_HOME, parent=SUBJECT_HOMES):
    """A fresh home for one session, as after a new Codex install.

    Codex injects `$CODEX_HOME/AGENTS.md` into every session and lists the
    skills of `~/.agents/skills` and of installed plugins; the operator's would
    steer subjects, lengthen every prompt and change between campaigns. The
    home holds only a copy of the credentials, and Codex installs its bundled
    system skills there. It sits outside the campaign, which is archived, and
    is retired afterwards.
    """
    parent.mkdir(mode=0o700, exist_ok=True)
    home = Path(tempfile.mkdtemp(dir=parent))
    try:
        credentials = home / ".codex/auth.json"
        credentials.parent.mkdir()
        shutil.copyfile(source / "auth.json", credentials)
        credentials.chmod(0o600)
        yield home
    finally:
        retire_home(home, source)


def retire_home(home, source=CODEX_HOME):
    """Remove a subject home, first writing back its credentials if they are newer.

    Codex rotates the refresh token when it refreshes, so the operator's copy
    would stop working if a refreshed one were discarded.
    """
    credentials = home / ".codex/auth.json"
    with CREDENTIALS:
        refreshed = lambda path: json.loads(path.read_text()).get("last_refresh", "")
        if credentials.is_file() and refreshed(credentials) > refreshed(source / "auth.json"):
            staged = source / f".auth.json.{os.getpid()}.{threading.get_ident()}"
            shutil.copyfile(credentials, staged)
            staged.chmod(0o600)
            os.replace(staged, source / "auth.json")
    shutil.rmtree(home)


def retire_leftover_homes(source=CODEX_HOME, parent=SUBJECT_HOMES):
    """Retire the homes an interrupted run left; a killed runner never reaches its cleanup.

    Campaigns run one at a time, so every home present before a run starts is a leftover.
    """
    for home in sorted(parent.iterdir()) if parent.is_dir() else []:
        retire_home(home, source)


def context_audit(rows):
    """What a session was given besides the task: injected instructions, skills and the model configuration.

    The fingerprint covers what every session of a campaign must share: the
    base instructions, the kind of each developer instruction, which Codex
    sends as the first line of a message item, and the skill names.
    Paths, the environment and the task differ per session and are left out.
    """
    contexts = [r["payload"] for r in rows if r["type"] == "turn_context"]
    messages = [r["payload"] for r in rows if r["type"] == "response_item" and r["payload"].get("role") in ["user", "developer"]]
    texts = [(m["role"], "".join(c.get("text", "") for c in m.get("content") or [])) for m in messages]
    skills = sorted(name for role, text in texts if text.startswith("<skills_instructions>")
                    for name in re.findall(r"^- ([^:\n`]+): .*\(file: ", text, re.MULTILINE))
    base = next((r["payload"].get("base_instructions") for r in rows if r["type"] == "session_meta"), None)
    kinds = [(c.get("text") or "").split("\n", 1)[0] for m in messages if m["role"] == "developer" for c in m.get("content") or []]
    shared = dict(base=base, developer=kinds, skills=skills)
    return dict(repository_context=any("Repository Guidelines" in text or "Agent Explorability Benchmark" in text for _, text in texts),
                global_instructions=any("# AGENTS.md instructions" in text for _, text in texts),
                configurations=[{"model": c.get("model"), "effort": c.get("effort")} for c in contexts],
                skills=skills, fingerprint=hashlib.sha256(json.dumps(shared, sort_keys=True).encode()).hexdigest())


def context_fingerprints(destination):
    """How many sessions of a campaign saw each context; an admissible campaign has one."""
    return dict(collections.Counter(json.loads(audit.read_text())["fingerprint"]
                                    for audit in destination.glob("*/evidence/*/context-audit.json")))


def run_agent(destination, run, model, effort):
    manifest = json.loads((destination / "manifest.json").read_text())
    task = manifest.get("task", json.loads(TASK.read_text()))
    condition = manifest["order"][run]
    evidence = destination / "evidence" / run
    events_path = evidence / "events.jsonl"
    if events_path.exists():
        raise ValueError("Refusing to overwrite a previous run; prepare a fresh campaign")
    executable = destination / condition / "target/debug/locmetry"
    generated_hashes = {}
    for item in task.get("generated", []):
        path = destination / run / item["path"]
        path.parent.mkdir(parents=True, exist_ok=True)
        result = subprocess.run([str(executable), *item["args"]], cwd=destination / run,
                                env=dict(os.environ, XDG_CONFIG_HOME=str(destination / "user-config" / run)),
                                capture_output=True, check=True)
        path.write_bytes(result.stdout)
        generated_hashes[item["path"]] = sha(path)
    invocation = [
        "codex", "exec", "--ignore-user-config", "--skip-git-repo-check",
        "--cd", str(destination / run), "--model", model,
        "-c", f'model_reasoning_effort="{effort}"', "-c", "project_doc_max_bytes=0",
        "--sandbox", "workspace-write", "--json", "--output-last-message",
        str(evidence / "answer.txt"), "-",
    ]
    settings = dict(command=invocation, binary_sha256=sha(executable), model=model, effort=effort,
                    prompt_sha256=sha(evidence / "prompt.txt"),
                    home="fresh, credentials only", started=time.time())
    (evidence / "invocation.json").write_text(json.dumps(settings, indent=2) + "\n")
    session = None
    stopped = None
    usage = {}
    tool_calls = 0
    thread = None
    written = None
    with subject_home() as home, (evidence / "prompt.txt").open() as prompt, events_path.open("w") as events, \
            (evidence / "stderr.txt").open("w") as stderr:
        process = subprocess.Popen(invocation, stdin=prompt, stdout=events, stderr=stderr, start_new_session=True,
                                   env=dict(os.environ, HOME=str(home), CODEX_HOME=str(home / ".codex")))
        while True:
            stream = records(events_path)
            if not thread:
                thread = next((r["thread_id"] for r in stream if r["type"] == "thread.started"), None)
            if thread and not session:
                session = next((home / ".codex/sessions").rglob(f"*{thread}*.jsonl"), None)
            if session:
                rows = records(session)
                audit = context_audit(rows)
                if audit != written:
                    (evidence / "context-audit.json").write_text(json.dumps(audit, indent=2) + "\n")
                    written = audit
                if audit["repository_context"]:
                    stopped = "repository-context-contamination"
                if audit["global_instructions"]:
                    stopped = "global-context-contamination"
                if any(c["model"] != model or c["effort"] != effort for c in audit["configurations"]):
                    stopped = "model-configuration-mismatch"
                counts = [info["total_token_usage"] for info in token_counts(rows)]
                usage = counts[-1] if counts else {}
                tool_calls = sum(r["type"] == "response_item" and r["payload"].get("type") in ["function_call", "custom_tool_call"] for r in rows)
            limits = manifest["limits"]
            if time.time() - settings["started"] > limits["wall_seconds"]:
                stopped = "wall-limit"
            if tool_calls > limits["agent_tool_calls"]:
                stopped = "tool-limit"
            if len(records(evidence / "calls.jsonl")) > limits["cli_invocations"]:
                stopped = "cli-limit"
            if usage.get("input_tokens", 0) - usage.get("cached_input_tokens", 0) > limits["uncached_input_tokens"]:
                stopped = "token-limit"
            if stopped and process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
            if process.poll() is not None:
                break
            time.sleep(1)
        # The home and the session in it are removed when the block ends.
        if session:
            shutil.copyfile(session, evidence / "session.jsonl")
    stream = records(events_path)
    final_usage = [r["usage"] for r in stream if r["type"] == "turn.completed"]
    if final_usage:
        usage = final_usage[-1]
    growth = {}
    if session:
        rows = records(evidence / "session.jsonl")
        tool_calls = sum(r["type"] == "response_item" and r["payload"].get("type") in ["function_call", "custom_tool_call"] for r in rows)
        growth = context_growth(rows)
    integrity = all((destination / run / p).is_file() and (destination / run / p).read_text() == content
                    for p, content in task["files"].items() if p not in task.get("mutable", []))
    integrity = integrity and all((destination / run / p).is_file() and sha(destination / run / p) == digest
                                  for p, digest in generated_hashes.items())
    summary = dict(condition=condition, run=run, thread_id=thread, session=str(session) if session else None,
                   elapsed=time.time()-settings["started"], returncode=process.returncode, stopped=stopped,
                   fixture_intact=integrity, usage=usage, agent_tool_calls=tool_calls,
                   shell_executions=sum(r["type"] == "item.completed" and r.get("item", {}).get("type") == "command_execution" for r in stream),
                   cli_invocations=len(records(evidence / "calls.jsonl")), **growth)
    (evidence / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


def campaign(destination, binaries, model, effort, treatment="without-guides", task_ids=(1, 2, 3, 4, 5, 6), repetitions=3):
    destination.mkdir(parents=True, exist_ok=False)
    tasks = {n: TASK.with_name(f"task{n}.json") for n in task_ids}
    jobs = [(n, repetition) for repetition in range(1, repetitions + 1) for n in task_ids]
    plan = dict(revision="six-tasks-v1" if treatment == "without-guides" else "help-recipe-v1",
                treatment=treatment, model=model, effort=effort, repetitions=repetitions,
                task_hashes={p.name: sha(p) for p in tasks.values()}, harness_sha256=sha(Path(__file__)),
                binary_hashes={c: sha(p) for c, p in zip(["A", "B"], binaries)},
                pairs=[dict(task=n, repetition=r, first="A" if (n+r)%2==0 else "B") for n, r in jobs],
                concurrent_pairs=3, previous_pilot_included=False)
    if treatment == "help-recipe":
        plan["recipe_sha256"] = sha(TASK.parent / "help-recipe.txt")
        plan["study_sha256"] = sha(TASK.parent / "help-recipe-study.json")
        plan["grader_sha256"] = sha(REPO / "scripts/score_agent_explorability.py")
    (destination / "campaign.json").write_text(json.dumps(plan, indent=2) + "\n")

    def pair(job):
        n, repetition = job
        target = destination / f"task{n}-rep{repetition}"
        prepare(target, tasks[n], binaries, treatment)
        order = ["run-02", "run-01"] if (n+repetition)%2==0 else ["run-01", "run-02"]
        for run in order:
            run_agent(target, run, model, effort)
        return target.name

    with ThreadPoolExecutor(max_workers=3) as pool:
        for done in pool.map(pair, jobs):
            print("Completed pair:", done, flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    prep.add_argument("destination", type=Path)
    prep.add_argument("--task", type=Path, default=TASK)
    prep.add_argument("--treatment", choices=["without-guides", "help-recipe", "help-routing", "supplied-binaries"], default="without-guides")
    prep.add_argument("--binaries", type=Path, nargs="+", metavar="BINARY")
    check = sub.add_parser("verify")
    check.add_argument("current", type=Path)
    check.add_argument("without_guides", type=Path)
    run = sub.add_parser("run")
    run.add_argument("destination", type=Path)
    run.add_argument("run", choices=["run-01", "run-02", "run-03"])
    run.add_argument("--model", required=True)
    run.add_argument("--effort", required=True)
    batch = sub.add_parser("campaign")
    batch.add_argument("destination", type=Path)
    batch.add_argument("--binaries", type=Path, nargs=2, required=True)
    batch.add_argument("--model", required=True)
    batch.add_argument("--effort", required=True)
    batch.add_argument("--treatment", choices=["without-guides", "help-recipe"], default="without-guides")
    batch.add_argument("--task-ids", type=int, nargs="+", default=[1, 2, 3, 4, 5, 6])
    batch.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.destination.resolve(), args.task.resolve(), args.binaries, args.treatment)
    elif args.command == "verify":
        verify(args.current.resolve(), args.without_guides.resolve())
    elif args.command == "run":
        run_agent(args.destination.resolve(), args.run, args.model, args.effort)
    else:
        campaign(args.destination.resolve(), [p.resolve() for p in args.binaries], args.model, args.effort,
                 args.treatment, args.task_ids, args.repetitions)


if __name__ == "__main__":
    main()
