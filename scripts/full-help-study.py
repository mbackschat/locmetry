"""Freeze and run the bounded full-help comparison using supplied native binaries."""

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path
import shutil
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
}


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


def campaign(destination, binaries, revisions, protocol_path=PROTOCOL, *, prepare_only=False):
    protocol = json.loads(protocol_path.read_text())
    validate_protocol(protocol)
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
    for item in protocol["pairs"]:
        target = destination / f"task{item['task']}-rep{item['repetition']}"
        task_path = frozen_tasks / task_paths[item["task"]].name
        BENCH.prepare(target, task_path, [frozen_binaries / arm for arm in "AB"],
                      "supplied-binaries", arm_provenance=arms)
        manifest = json.loads((target / "manifest.json").read_text())
        if snapshot is not None and snapshot != manifest["repository_snapshot"]:
            raise ValueError("The preparation repository changed while freezing the study")
        snapshot = manifest["repository_snapshot"]
    plan = dict(protocol, treatment="supplied-binaries", arms=arms, repository_snapshot=snapshot,
                protocol_sha256=BENCH.sha(destination / "protocol.json"),
                runner_sha256=BENCH.sha(Path(__file__)), harness_sha256=BENCH.sha(Path(BENCH.__file__)),
                grader_sha256=BENCH.sha(BENCH.REPO / "scripts/score_agent_explorability.py"),
                full_help_grader_sha256=BENCH.sha(BENCH.REPO / "scripts/score_full_help.py"),
                task_hashes={str(n): BENCH.sha(frozen_tasks / p.name) for n, p in task_paths.items()},
                frozen_files={str(p.relative_to(destination)): BENCH.sha(p)
                              for p in sorted(destination.rglob("*")) if p.is_file()})
    (destination / "campaign.json").write_text(json.dumps(plan, indent=2) + "\n")
    (destination / "campaign.sha256").write_text(BENCH.sha(destination / "campaign.json") + "\n")
    if not prepare_only:
        run_frozen(destination)


def run_frozen(destination):
    if (destination / "started.json").exists():
        raise ValueError("This bounded campaign already started; no replacement or extra trials are permitted")
    if BENCH.sha(destination / "campaign.json") != (destination / "campaign.sha256").read_text().strip():
        raise ValueError("Frozen campaign manifest changed")
    plan = json.loads((destination / "campaign.json").read_text())
    validate_protocol(plan)
    for name, expected in plan["frozen_files"].items():
        path = destination / name
        if not path.is_file() or BENCH.sha(path) != expected:
            raise ValueError(f"Frozen input changed: {name}")
    for path, expected in [(Path(__file__), plan["runner_sha256"]),
                           (Path(BENCH.__file__), plan["harness_sha256"]),
                           (BENCH.REPO / "scripts/score_agent_explorability.py", plan["grader_sha256"]),
                           (BENCH.REPO / "scripts/score_full_help.py", plan["full_help_grader_sha256"])]:
        if BENCH.sha(path) != expected:
            raise ValueError(f"Study code changed after freezing: {path.name}")
    with (destination / "started.json").open("x") as started:
        json.dump({"started": time.time(), "sessions": len(plan["pairs"]) * 2,
                   "protocol_sha256": plan["protocol_sha256"]}, started, indent=2)
        started.write("\n")

    def pair(item):
        target = destination / f"task{item['task']}-rep{item['repetition']}"
        for arm in item["order"]:
            BENCH.run_agent(target, {"A": "run-02", "B": "run-01"}[arm], plan["model"], plan["effort"])
        return target.name

    with ThreadPoolExecutor(max_workers=plan["concurrent_pairs"]) as pool:
        for done in pool.map(pair, plan["pairs"]):
            print("Completed pair:", done, flush=True)


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
    args = parser.parse_args()
    supplied = [args.control, args.candidate, args.control_revision, args.candidate_revision]
    if args.run_frozen:
        if any(supplied):
            parser.error("--run-frozen uses the already frozen binaries and revisions")
        run_frozen(args.destination.resolve())
    else:
        if not all(supplied):
            parser.error("--control, --candidate, --control-revision and --candidate-revision are required")
        campaign(args.destination.resolve(), [args.control.resolve(), args.candidate.resolve()],
                 [args.control_revision, args.candidate_revision], args.protocol.resolve(),
                 prepare_only=args.prepare_only)


if __name__ == "__main__":
    main()
