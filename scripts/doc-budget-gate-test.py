import importlib.util
import json
import subprocess
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

SPEC = importlib.util.spec_from_file_location(
    "gate", Path(__file__).with_name("doc-budget-gate.py"))
gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gate)


def summary(findings=0, undecided=0):
    warnings = int(bool(findings or undecided))
    return dict(record="lint_summary", v=3, root=".",
                scope="recursive-directory", max_warning_files="0", files=1,
                errors=0, warning_files=warnings, warning_files_shown=0,
                warning_files_hidden=warnings, findings=findings,
                findings_shown=0, findings_hidden=findings,
                undecided=undecided, undecided_shown=0,
                undecided_hidden=undecided, overlong_doc_findings=findings,
                overlong_doc_undecided=undecided, over_budget=findings,
                configuration_dependent=0, unreadable_doc_payload=undecided,
                uninspected_macro_body=0)


def result(data, code=0, stdout="", extra=""):
    return subprocess.CompletedProcess([], code, stdout, json.dumps(data) + "\n" + extra)


class GateTests(unittest.TestCase):
    def test_clean_and_hidden_findings(self):
        self.assertEqual(gate.classify(result(summary()))[0], 0)
        self.assertEqual(gate.classify(result(summary(23), 4))[0], 1)

    def test_undecided_and_operational_are_unknown(self):
        for findings in (0, 3):
            self.assertEqual(gate.classify(result(summary(findings, 1), 4))[0], 2)
        for code in (1, 2, 3, 5, -9):
            self.assertEqual(gate.classify(result(summary(), code))[0], 2)

    def test_invalid_protocol_is_unknown(self):
        for key, value in (("v", 2), ("v", True), ("files", 0),
                           ("findings", True), ("errors", -1),
                           ("undecided", "0"), ("findings_hidden", 1),
                           ("over_budget", 1), ("warning_files", 2),
                           ("max_warning_files", "1"), ("scope", "file")):
            data = summary()
            data[key] = value
            with self.subTest(key=key, value=value):
                self.assertEqual(gate.classify(result(data))[0], 2)
        for data in (None, [], {}, dict(summary(), unexpected=1)):
            self.assertEqual(gate.classify(result(data))[0], 2)
        for extra in (json.dumps(summary()), '{broken', 'unexpected diagnostic',
                      '{"record":"run_error","v":3}'):
            self.assertEqual(gate.classify(result(summary(), extra=extra))[0], 2)
        self.assertEqual(gate.classify(result(summary(), stdout="unexpected"))[0], 2)
        self.assertEqual(gate.classify(result(summary(), 4))[0], 2)
        self.assertEqual(gate.classify(result(summary(1), 0))[0], 2)
        duplicate = json.dumps(summary()).replace('"v": 3', '"v": 2, "v": 3')
        self.assertEqual(gate.classify(subprocess.CompletedProcess([], 0, "", duplicate))[0], 2)

    def test_real_boundaries_and_undecided(self):
        with TemporaryDirectory(prefix=".doc-budget-test-", dir=Path(__file__).parent) as root:
            source = Path(root) / "lib.rs"
            for words, expected in ((80, 0), (81, 0), (120, 0), (121, 1)):
                source.write_text("/// " + " ".join(["word"] * words) + "\npub struct Probe;\n")
                with self.subTest(words=words):
                    self.assertEqual(gate.main(root), expected)
            source.write_text('#[doc = include_str!("absent.md")]\npub struct Probe;\n')
            self.assertEqual(gate.main(root), 2)


if __name__ == "__main__":
    unittest.main()
