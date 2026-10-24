"""Freeze and run the bounded full-help comparison using supplied native binaries."""

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time


SPEC = importlib.util.spec_from_file_location("benchmark", Path(__file__).with_name("agent-explorability.py"))
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)
PROTOCOL = BENCH.TASK.parent / "full-help-study.json"
TASK_IDS = [1, 2, 3, 4, 5, 6, 7, 9, 10, 11]
# Each frozen protocol's task set; a new study adds its own revision here.
STUDY_TASKS = {
    "full-help-coverage-v1": TASK_IDS,
    "compare-help-routing-v1": [1, 4, 5, 7],
    "agent-regression-0.8-v1": [3, 4, 8],
    "agent-regression-0.8-correction-v1": [4],
    "agent-regression-2026-10-23-v1": [3, 8],
    "agent-regression-2026-10-24-v1": [3, 8, 12],
    "agent-regression-2026-10-24-correction-v1": [12],
    "agent-regression-2026-10-24-sections-v1": [1, 3, 8, 12],
    "agent-regression-2026-10-24-sections-v2": [1, 3, 8, 12],
    "agent-regression-2026-10-24-sections-correction-v1": [1],
}


# Codex's workspace-write sandbox lets an explorer write only below its run
# directory, /tmp and $TMPDIR, and each run's launcher logs calls in the
# evidence folder beside it. A campaign frozen elsewhere recorded no CLI call
# in any of its 24 sessions (0.8.0 regression run, 2026-10-22), so campaigns
# run under a temporary root and are archived afterwards.
EXPLORER_WRITABLE_ROOTS = (Path("/tmp").resolve(), Path(tempfile.gettempdir()).resolve())


def require_explorer_writable(destination):
    if not any(destination.resolve().is_relative_to(root) for root in EXPLORER_WRITABLE_ROOTS):
        raise ValueError(f"Campaigns must run below {' or '.join(map(str, EXPLORER_WRITABLE_ROOTS)) or 'a temporary root'}, "
                         "where the explorer sandbox lets the launcher record calls; archive the evidence afterwards")


def codex_catalog(model, effort):
    """Codex's own catalog entry for the frozen model, refusing one it does not offer.

    The owner's tier mapping names the model, and Codex cannot tell which model is
    a tier's latest; it can confirm the model and effort exist and name newer
    versions of the same family, which the owner then weighs before acknowledging.
    """
    version = subprocess.run(["codex", "--version"], capture_output=True, text=True, check=True).stdout.strip()
    models = json.loads(subprocess.run(["codex", "debug", "models"], capture_output=True, text=True, check=True).stdout)["models"]
    entry = next((m for m in models if m["slug"] == model), None)
    if entry is None:
        raise ValueError(f"Codex {version} does not list model {model}")
    efforts = [level["effort"] for level in entry.get("supported_reasoning_levels", [])]
    if effort not in efforts:
        raise ValueError(f"Codex {version} does not support effort {effort} for {model}: {', '.join(efforts)}")

    def family(slug):
        match = re.fullmatch(r"(.+?)-(\d+(?:\.\d+)*)-([a-z]+)", slug)
        return match and (match[1], match[3], tuple(int(part) for part in match[2].split(".")))

    own = family(model)
    newer = sorted(m["slug"] for m in models
                   if own and family(m["slug"]) and family(m["slug"])[:2] == own[:2] and family(m["slug"])[2] > own[2])
    return {"codex_version": version, "model": model, "efforts": efforts, "newer_in_family": newer}


def require_acknowledgement(plan, acknowledgement):
    expected = f"{plan['model']}:{plan['effort']}"
    if acknowledgement != expected:
        raise ValueError(f"The owner must acknowledge the frozen model and effort before launch: --acknowledge {expected}")


def validate_protocol(protocol):
    task_ids = STUDY_TASKS.get(protocol["revision"])
    if task_ids is None:
        raise ValueError("Unknown study; only the fixed protocols in STUDY_TASKS are supported")
    if protocol["task_ids"] != task_ids or protocol["repetitions"] != 4 or protocol["concurrent_pairs"] != 3:
        raise ValueError("The fixed protocol requires its task set, four pairs each and at most three concurrent pairs")
    pairs = protocol["pairs"]
    expected_pairs = len(task_ids) * 4
    if len(pairs) != expected_pairs or protocol["maximum_sessions"] != expected_pairs * 2:
        raise ValueError(f"The bounded study requires exactly {expected_pairs} pairs ({expected_pairs * 2} sessions)")
    for task in task_ids:
        selected = [p for p in pairs if p["task"] == task]
        if {p["repetition"] for p in selected} != {1, 2, 3, 4} or Counter(p["order"] for p in selected) != {"AB": 2, "BA": 2}:
            raise ValueError(f"Task {task} requires AB/BA twice each")
    # A predeclared confirmation sample: four more balanced pairs per task, frozen
    # with the primary sample and run at most once (docs/agent-regression-checks.md).
    confirmation = protocol.get("confirmation")
    if confirmation is not None:
        extra = confirmation["pairs"]
        if len(extra) != len(task_ids) * 4 or confirmation["maximum_sessions"] != len(extra) * 2:
            raise ValueError("A confirmation sample requires four pairs per task")
        for task in task_ids:
            selected = [p for p in extra if p["task"] == task]
            if {p["repetition"] for p in selected} != {5, 6, 7, 8} or Counter(p["order"] for p in selected) != {"AB": 2, "BA": 2}:
                raise ValueError(f"Task {task} requires confirmation pairs 5-8, AB/BA twice each")


def all_pairs(protocol):
    return protocol["pairs"] + protocol.get("confirmation", {}).get("pairs", [])


def campaign(destination, binaries, revisions, protocol_path=PROTOCOL, *, prepare_only=False, acknowledgement=None):
    require_explorer_writable(destination)
    protocol = json.loads(protocol_path.read_text())
    validate_protocol(protocol)
    if not prepare_only:
        require_acknowledgement(protocol, acknowledgement)
    catalog = codex_catalog(protocol["model"], protocol["effort"])
    if len(binaries) != 2 or len(revisions) != 2 or not all(revisions):
        raise ValueError("Supply both binaries and their source revisions")
    for binary in binaries:
        if not binary.is_file():
            raise FileNotFoundError(binary)
    expected = protocol["control"]
    if revisions[0] != expected["source_revision"] or BENCH.sha(binaries[0]) != expected["binary_sha256"]:
        raise ValueError("The control revision/binary does not match the frozen control provenance")
    task_paths = {
        number: BENCH.TASK.with_name(protocol["task_files"].get(str(number), f"task{number}.json"))
        for number in protocol["task_ids"]
    }
    for path in task_paths.values():
        if not path.is_file():
            raise FileNotFoundError(path)
        if json.loads(path.read_text())["limits"] != protocol["limits"]:
            raise ValueError(f"Task budgets differ from the frozen protocol: {path.name}")
    destination.mkdir(parents=True, exist_ok=False)
    shutil.copyfile(protocol_path, destination / "protocol.json")
    frozen_tasks = destination / "tasks"
    frozen_tasks.mkdir()
    for path in task_paths.values():
        shutil.copyfile(path, frozen_tasks / path.name)
    frozen_binaries = destination / "binaries"
    frozen_binaries.mkdir()
    for arm, binary in zip("AB", binaries):
        shutil.copy2(binary, frozen_binaries / arm)
        # Every pair hard-links this file, so none may change it in place.
        (frozen_binaries / arm).chmod(0o555)
    scripts = destination / "scripts"
    scripts.mkdir()
    for path in [Path(__file__), Path(BENCH.__file__), BENCH.REPO / "scripts/score_agent_explorability.py",
                 BENCH.REPO / "scripts/score_full_help.py"]:
        shutil.copyfile(path, scripts / path.name)
    arms = {
        arm: {"source_revision": revision, "source_provenance": "operator-declared prebuilt binary",
              "binary_sha256": BENCH.sha(frozen_binaries / arm)}
        for arm, revision in zip("AB", revisions)
    }
    snapshot = None
    for item in all_pairs(protocol):
        target = destination / f"task{item['task']}-rep{item['repetition']}"
        task_path = frozen_tasks / task_paths[item["task"]].name
        BENCH.prepare(target, task_path, [frozen_binaries / arm for arm in "AB"],
                      "supplied-binaries", arm_provenance=arms)
        manifest = json.loads((target / "manifest.json").read_text())
        if snapshot is not None and snapshot != manifest["repository_snapshot"]:
            raise ValueError("The preparation repository changed while freezing the study")
        snapshot = manifest["repository_snapshot"]
    plan = dict(protocol, treatment="supplied-binaries", arms=arms, repository_snapshot=snapshot, codex_catalog=catalog,
                protocol_sha256=BENCH.sha(destination / "protocol.json"),
                runner_sha256=BENCH.sha(Path(__file__)), harness_sha256=BENCH.sha(Path(BENCH.__file__)),
                grader_sha256=BENCH.sha(BENCH.REPO / "scripts/score_agent_explorability.py"),
                full_help_grader_sha256=BENCH.sha(BENCH.REPO / "scripts/score_full_help.py"),
                task_hashes={str(n): BENCH.sha(frozen_tasks / p.name) for n, p in task_paths.items()},
                frozen_files={str(p.relative_to(destination)): BENCH.sha(p)
                              for p in sorted(destination.rglob("*")) if p.is_file()})
    (destination / "campaign.json").write_text(json.dumps(plan, indent=2) + "\n")
    (destination / "campaign.sha256").write_text(BENCH.sha(destination / "campaign.json") + "\n")
    print(f"Frozen for {catalog['model']} at {protocol['effort']} ({catalog['codex_version']}); "
          f"newer in its family: {', '.join(catalog['newer_in_family']) or 'none'}", flush=True)
    if not prepare_only:
        run_frozen(destination, acknowledgement)


def frozen_plan(destination, acknowledgement, *, ran=()):
    """The frozen campaign plan, after checking that nothing it froze has changed.

    Pairs that already ran are skipped: their sessions may edit the files a
    task declares mutable.
    """
    require_explorer_writable(destination)
    if BENCH.sha(destination / "campaign.json") != (destination / "campaign.sha256").read_text().strip():
        raise ValueError("Frozen campaign manifest changed")
    plan = json.loads((destination / "campaign.json").read_text())
    validate_protocol(plan)
    for name, expected in plan["frozen_files"].items():
        if Path(name).parts[0] in ran:
            continue
        path = destination / name
        if not path.is_file() or BENCH.sha(path) != expected:
            raise ValueError(f"Frozen input changed: {name}")
    for path, expected in [(Path(__file__), plan["runner_sha256"]),
                           (Path(BENCH.__file__), plan["harness_sha256"]),
                           (BENCH.REPO / "scripts/score_agent_explorability.py", plan["grader_sha256"]),
                           (BENCH.REPO / "scripts/score_full_help.py", plan["full_help_grader_sha256"])]:
        if BENCH.sha(path) != expected:
            raise ValueError(f"Study code changed after freezing: {path.name}")
    require_acknowledgement(plan, acknowledgement)
    return plan


def run_pairs(destination, plan, pairs, marker, acknowledgement):
    catalog = codex_catalog(plan["model"], plan["effort"])
    BENCH.retire_leftover_homes()
    with (destination / marker).open("x") as started:
        json.dump({"started": time.time(), "sessions": len(pairs) * 2,
                   "protocol_sha256": plan["protocol_sha256"], "acknowledged": acknowledgement,
                   "codex_catalog": catalog}, started, indent=2)
        started.write("\n")

    def pair(item):
        target = destination / f"task{item['task']}-rep{item['repetition']}"
        for arm in item["order"]:
            BENCH.run_agent(target, {"A": "run-02", "B": "run-01"}[arm], plan["model"], plan["effort"])
        return target.name

    with ThreadPoolExecutor(max_workers=plan["concurrent_pairs"]) as pool:
        for done in pool.map(pair, pairs):
            print("Completed pair:", done, flush=True)


def run_frozen(destination, acknowledgement):
    if (destination / "started.json").exists():
        raise ValueError("This bounded campaign already started; no replacement or extra trials are permitted")
    plan = frozen_plan(destination, acknowledgement)
    run_pairs(destination, plan, plan["pairs"], "started.json", acknowledgement)
    (destination / "completed.json").write_text(json.dumps(
        {"completed": time.time(), "context_fingerprints": BENCH.context_fingerprints(destination)}) + "\n")


def run_confirmation(destination, acknowledgement, tasks):
    """Runs the frozen confirmation pairs of the named tasks, once per campaign.

    The protocol's activation and decision rules decide whether to call this;
    the runner only enforces that the primary sample finished and that it runs
    at most once.
    """
    if not (destination / "completed.json").exists():
        raise ValueError("The confirmation sample runs only after the primary sample completed")
    if (destination / "confirmation-started.json").exists():
        raise ValueError("The confirmation sample already ran; no further trials are permitted")
    primary = json.loads((destination / "campaign.json").read_text())["pairs"]
    plan = frozen_plan(destination, acknowledgement, ran={f"task{p['task']}-rep{p['repetition']}" for p in primary})
    unknown = sorted(set(tasks) - set(plan["task_ids"]))
    if unknown or "confirmation" not in plan:
        raise ValueError(f"Confirmation tasks not in this protocol: {unknown or 'no confirmation sample'}")
    pairs = [p for p in plan["confirmation"]["pairs"] if p["task"] in tasks]
    run_pairs(destination, plan, pairs, "confirmation-started.json", acknowledgement)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--control", type=Path)
    parser.add_argument("--candidate", type=Path)
    parser.add_argument("--control-revision")
    parser.add_argument("--candidate-revision")
    parser.add_argument("--protocol", type=Path, default=PROTOCOL)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--prepare-only", action="store_true")
    mode.add_argument("--run-frozen", action="store_true")
    mode.add_argument("--run-confirmation", type=int, nargs="+", metavar="TASK",
                      help="run the frozen confirmation pairs of these tasks, once, when the protocol's activation rule holds")
    parser.add_argument("--acknowledge", metavar="MODEL:EFFORT",
                        help="the owner's acknowledgement of the frozen model and effort; required to launch")
    args = parser.parse_args()
    supplied = [args.control, args.candidate, args.control_revision, args.candidate_revision]
    if args.run_frozen or args.run_confirmation:
        if any(supplied):
            parser.error("--run-frozen and --run-confirmation use the already frozen binaries and revisions")
        if args.run_confirmation:
            run_confirmation(args.destination.resolve(), args.acknowledge, args.run_confirmation)
        else:
            run_frozen(args.destination.resolve(), args.acknowledge)
    else:
        if not all(supplied):
            parser.error("--control, --candidate, --control-revision and --candidate-revision are required")
        campaign(args.destination.resolve(), [args.control.resolve(), args.candidate.resolve()],
                 [args.control_revision, args.candidate_revision], args.protocol.resolve(),
                 prepare_only=args.prepare_only, acknowledgement=args.acknowledge)


if __name__ == "__main__":
    main()
