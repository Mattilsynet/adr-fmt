import json
import subprocess
import sys
from pathlib import Path


COUNTERS = set("files errors warning_files warning_files_shown warning_files_hidden "
               "findings findings_shown findings_hidden undecided undecided_shown "
               "undecided_hidden overlong_doc_findings overlong_doc_undecided "
               "over_budget configuration_dependent unreadable_doc_payload "
               "uninspected_macro_body".split())


def unique_object(pairs):
    obj = {}
    for key, value in pairs:
        if key in obj:
            raise ValueError("duplicate key")
        obj[key] = value
    return obj


def classify(result):
    try:
        if result.returncode not in (0, 4) or result.stdout.strip():
            raise ValueError("producer failed or unexpected detail output")
        lines = result.stderr.splitlines()
        diagnostic = "error: doc lint did not establish a clean tree"
        if lines and lines[-1] == diagnostic and result.returncode == 4:
            lines.pop()
        if len(lines) != 1:
            raise ValueError("expected exactly one summary")
        s = json.loads(lines[0], object_pairs_hook=unique_object)
        if not isinstance(s, dict) or set(s) != COUNTERS | {
                "record", "v", "root", "scope", "max_warning_files"}:
            raise ValueError("unsupported summary shape")
        if (s["record"] != "lint_summary" or type(s["v"]) is not int or s["v"] != 3
                or s["scope"] != "recursive-directory" or s["max_warning_files"] != "0"
                or not isinstance(s["root"], str) or not s["root"]):
            raise ValueError("unsupported protocol")
        if any(type(s[k]) is not int or s[k] < 0 for k in COUNTERS):
            raise ValueError("invalid counter")
        for family in ("warning_files", "findings", "undecided"):
            if s[family + "_shown"] != 0 or s[family + "_hidden"] != s[family]:
                raise ValueError("inconsistent detail totals")
        if (s["files"] == 0 or s["errors"] != 0
                or s["warning_files"] > s["files"]
                or s["warning_files"] > s["findings"] + s["undecided"]
                or bool(s["warning_files"]) != bool(s["findings"] + s["undecided"])
                or s["findings"] != s["overlong_doc_findings"]
                or s["findings"] != s["over_budget"]
                or s["undecided"] != s["overlong_doc_undecided"]
                or s["undecided"] != sum(s[k] for k in (
                    "configuration_dependent", "unreadable_doc_payload", "uninspected_macro_body"))
                or result.returncode != (4 if s["findings"] + s["undecided"] else 0)):
            raise ValueError("incomplete or inconsistent coverage")
        return (2 if s["undecided"] else int(bool(s["findings"]))), s
    except (ValueError, TypeError) as error:
        print(f"doc-budget: no verdict: {error}", file=sys.stderr)
        return 2, None


def main(root=None):
    root = root or str(Path(__file__).resolve().parents[1])
    outcomes = []
    for budget in (80, 120):
        try:
            result = subprocess.run(
                ["comment-free", "--doc-max-words", str(budget),
                 "--max-warning-files", "0", root],
                capture_output=True, text=True, check=False,
            )
        except (OSError, UnicodeError) as error:
            print(f"doc-budget: no verdict: {error}", file=sys.stderr)
            return 2
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)
        verdict, summary = classify(result)
        if summary is not None:
            print(f"doc-budget: budget={budget} findings={summary['findings']} "
                  f"undecided={summary['undecided']} errors={summary['errors']}")
        outcomes.append(0 if budget == 80 and verdict == 1 else verdict)
    return 2 if 2 in outcomes else max(outcomes)


if __name__ == "__main__":
    sys.exit(main())
