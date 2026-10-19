"""Prepare and check the task1-pilot-v1 experiment; never edit product sources."""

import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time


REPO = Path(__file__).resolve().parent.parent
TASK = REPO / "tests/fixtures/agent-explorability/task1.json"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def seed(root):
    for name, contents in json.loads(TASK.read_text())["files"].items():
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


def prepare(destination):
    destination.mkdir(parents=True, exist_ok=False)
    files = [REPO / "Cargo.toml", REPO / "Cargo.lock"]
    files += list((REPO / "src").rglob("*.rs"))
    files += list((REPO / "docs/guides").glob("*.md"))
    for condition in ["A", "B"]:
        for path in files:
            target = destination / condition / path.relative_to(REPO)
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, target)
    without_guides(destination / "B")
    task = json.loads(TASK.read_text())
    manifest = {
        "revision": task["revision"],
        "source_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "task_sha256": sha(TASK),
        "harness_sha256": sha(Path(__file__)),
        "source_hashes": {str(p.relative_to(REPO)): sha(p) for p in files},
        "limits": task["limits"],
        "order": {"run-01": "B", "run-02": "A"},
        "environment": {"COLUMNS": "100", "NO_COLOR": "1", "LC_ALL": "C", "stdout_tty": False},
    }
    for run, condition in manifest["order"].items():
        root = destination / run
        seed(root)
        (root / "bin").mkdir()
        logs = destination / "evidence" / run
        logs.mkdir(parents=True)
        config = destination / "user-config" / run
        config.mkdir(parents=True)
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
    for line in path.read_text().splitlines():
        try:
            result.append(json.loads(line))
        except json.JSONDecodeError:
            # An active writer may not yet have finished the last JSONL record.
            continue
    return result


def run_agent(destination, run, model, effort):
    manifest = json.loads((destination / "manifest.json").read_text())
    condition = manifest["order"][run]
    evidence = destination / "evidence" / run
    events_path = evidence / "events.jsonl"
    if events_path.exists():
        raise ValueError("Refusing to overwrite a previous run; prepare a fresh campaign")
    executable = destination / condition / "target/debug/locmetry"
    invocation = [
        "codex", "exec", "--ignore-user-config", "--skip-git-repo-check",
        "--cd", str(destination / run), "--model", model,
        "-c", f'model_reasoning_effort="{effort}"', "-c", "project_doc_max_bytes=0",
        "--sandbox", "workspace-write", "--json", "--output-last-message",
        str(evidence / "answer.txt"), "-",
    ]
    settings = dict(command=invocation, binary_sha256=sha(executable), model=model, effort=effort,
                    prompt_sha256=sha(evidence / "prompt.txt"), started=time.time())
    (evidence / "invocation.json").write_text(json.dumps(settings, indent=2) + "\n")
    session = None
    stopped = None
    usage = {}
    tool_calls = 0
    thread = None
    with (evidence / "prompt.txt").open() as prompt, events_path.open("w") as events, (evidence / "stderr.txt").open("w") as stderr:
        process = subprocess.Popen(invocation, stdin=prompt, stdout=events, stderr=stderr, start_new_session=True)
        while True:
            stream = records(events_path)
            if not thread:
                thread = next((r["thread_id"] for r in stream if r["type"] == "thread.started"), None)
            if thread and not session:
                session = next((Path.home() / ".codex/sessions").rglob(f"*{thread}*.jsonl"), None)
            if session:
                rows = records(session)
                contexts = [r["payload"] for r in rows if r["type"] == "turn_context"]
                messages = [r["payload"] for r in rows if r["type"] == "response_item" and r["payload"].get("role") in ["user", "developer"]]
                injected = any("Repository Guidelines" in json.dumps(m) or "Agent Explorability Benchmark" in json.dumps(m) for m in messages)
                actual = [{"model": c.get("model"), "effort": c.get("effort")} for c in contexts]
                (evidence / "context-audit.json").write_text(json.dumps(dict(repository_context=injected, configurations=actual), indent=2) + "\n")
                if injected:
                    stopped = "repository-context-contamination"
                if any(c["model"] != model or c["effort"] != effort for c in actual):
                    stopped = "model-configuration-mismatch"
                counts = [r["payload"]["info"]["total_token_usage"] for r in rows if r["type"] == "event_msg" and r["payload"].get("type") == "token_count" and r["payload"].get("info")]
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
    stream = records(events_path)
    final_usage = [r["usage"] for r in stream if r["type"] == "turn.completed"]
    if final_usage:
        usage = final_usage[-1]
    integrity = all((destination / run / p).read_text() == content for p, content in json.loads(TASK.read_text())["files"].items())
    summary = dict(condition=condition, run=run, thread_id=thread, session=str(session) if session else None,
                   elapsed=time.time()-settings["started"], returncode=process.returncode, stopped=stopped,
                   fixture_intact=integrity, usage=usage, agent_tool_calls=tool_calls,
                   shell_executions=sum(r["type"] == "item.completed" and r.get("item", {}).get("type") == "command_execution" for r in stream),
                   cli_invocations=len(records(evidence / "calls.jsonl")))
    (evidence / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    prep = sub.add_parser("prepare")
    prep.add_argument("destination", type=Path)
    check = sub.add_parser("verify")
    check.add_argument("current", type=Path)
    check.add_argument("without_guides", type=Path)
    run = sub.add_parser("run")
    run.add_argument("destination", type=Path)
    run.add_argument("run", choices=["run-01", "run-02"])
    run.add_argument("--model", required=True)
    run.add_argument("--effort", required=True)
    args = parser.parse_args()
    if args.command == "prepare":
        prepare(args.destination.resolve())
    elif args.command == "verify":
        verify(args.current.resolve(), args.without_guides.resolve())
    else:
        run_agent(args.destination.resolve(), args.run, args.model, args.effort)


if __name__ == "__main__":
    main()
