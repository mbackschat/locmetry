"""Run the frozen three-condition help-routing study in fresh scratch directories."""

import argparse
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path

SPEC = importlib.util.spec_from_file_location("benchmark", Path(__file__).with_name("agent-explorability.py"))
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)
PROTOCOL = BENCH.TASK.parent / "help-routing-study.json"


def campaign(destination, binaries):
    protocol = json.loads(PROTOCOL.read_text())
    if len(binaries) != 3:
        raise ValueError("Supply the current, example-only and routed-example binaries")
    destination.mkdir(parents=True, exist_ok=False)
    plan = dict(protocol, treatment="help-routing", protocol_sha256=BENCH.sha(PROTOCOL),
                runner_sha256=BENCH.sha(Path(__file__)), harness_sha256=BENCH.sha(Path(BENCH.__file__)),
                grader_sha256=BENCH.sha(BENCH.REPO / "scripts/score_agent_explorability.py"),
                recipe_sha256=BENCH.sha(BENCH.TASK.parent / "help-recipe.txt"),
                routing_sha256=BENCH.sha(BENCH.TASK.parent / "help-routing.txt"),
                task_hashes={str(n): BENCH.sha(BENCH.TASK.with_name(f"task{n}.json")) for n in protocol["task_ids"]},
                binary_hashes={arm: BENCH.sha(binary) for arm, binary in zip("ABC", binaries)})
    (destination / "campaign.json").write_text(json.dumps(plan, indent=2) + "\n")

    def triad(item):
        target = destination / f"task{item['task']}-rep{item['repetition']}"
        BENCH.prepare(target, BENCH.TASK.with_name(f"task{item['task']}.json"), binaries, "help-routing")
        runs = {"A": "run-02", "B": "run-01", "C": "run-03"}
        for arm in item["order"]:
            BENCH.run_agent(target, runs[arm], protocol["model"], protocol["effort"])
        return target.name

    with ThreadPoolExecutor(max_workers=protocol["concurrent_triads"]) as pool:
        for done in pool.map(triad, protocol["triads"]):
            print("Completed triad:", done, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--binaries", type=Path, nargs=3, required=True, metavar=("A", "B", "C"))
    args = parser.parse_args()
    campaign(args.destination.resolve(), [p.resolve() for p in args.binaries])
