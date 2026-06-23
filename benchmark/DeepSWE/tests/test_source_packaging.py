import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

from benchmark_deepswe.daat_locus_agent import (
    daat_locus_source_files,
    squid_safe_allowlist_domains,
)


@unittest.skipUnless(shutil.which("git"), "git is required for source packaging")
class SourcePackagingTests(unittest.TestCase):
    def test_gitignore_is_respected_while_including_local_sources(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            subprocess.run(["git", "init"], cwd=root, check=True, stdout=subprocess.PIPE)
            subprocess.run(
                ["git", "config", "core.autocrlf", "false"],
                cwd=root,
                check=True,
            )
            (root / ".gitignore").write_text("ignored.txt\n", encoding="utf-8")
            (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
            (root / "local.txt").write_text("local\n", encoding="utf-8")
            (root / "ignored.txt").write_text("ignored\n", encoding="utf-8")
            subprocess.run(["git", "add", "tracked.txt"], cwd=root, check=True)

            files = {path.as_posix() for path in daat_locus_source_files(root)}

            self.assertIn("tracked.txt", files)
            self.assertIn("local.txt", files)
            self.assertIn(".gitignore", files)
            self.assertNotIn("ignored.txt", files)

    def test_squid_allowlist_drops_duplicate_subdomain_entries_and_local_hosts(self):
        domains = squid_safe_allowlist_domains(
            [
                ".chatgpt.com",
                "chatgpt.com",
                ".models.dev",
                "models.dev.",
                "127.0.0.1",
                "::1",
                "localhost",
                ".openai.com",
            ]
        )

        self.assertEqual(
            domains,
            [".openai.com", "chatgpt.com", "models.dev"],
        )


if __name__ == "__main__":
    unittest.main()
