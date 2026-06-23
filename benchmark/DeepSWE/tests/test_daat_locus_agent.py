import ast
import unittest
from pathlib import Path

from benchmark_deepswe.daat_locus_agent import DaatLocusAgent


class FakeEnvironment:
    def __init__(self):
        self.received_env = None

    def agent_process_env(self, env):
        self.received_env = dict(env)
        merged = dict(env)
        merged["HTTPS_PROXY"] = "http://agent:token@pier-egress-proxy:8080"
        return merged


class DaatLocusAgentTests(unittest.TestCase):
    def test_agent_env_passes_through_pier_agent_process_env(self):
        environment = FakeEnvironment()
        agent = DaatLocusAgent(
            logs_dir=Path("logs"),
            extra_env={"EXTRA": "value"},
            forward_env="",
            container_home="/tmp/daat-home",
        )

        env = agent._agent_env(environment, {"LOCAL": "value"})

        self.assertEqual(environment.received_env["DAAT_LOCUS_HOME"], "/tmp/daat-home")
        self.assertEqual(environment.received_env["EXTRA"], "value")
        self.assertEqual(environment.received_env["LOCAL"], "value")
        self.assertEqual(
            env["HTTPS_PROXY"],
            "http://agent:token@pier-egress-proxy:8080",
        )

    def test_environment_exec_calls_explicitly_pass_env(self):
        source = (
            Path(__file__).parents[1]
            / "src"
            / "benchmark_deepswe"
            / "daat_locus_agent.py"
        )
        tree = ast.parse(source.read_text(encoding="utf-8"))
        missing_env_lines = []
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            if not isinstance(node.func, ast.Attribute):
                continue
            if node.func.attr != "exec":
                continue
            if not isinstance(node.func.value, ast.Name):
                continue
            if node.func.value.id != "environment":
                continue
            if not any(keyword.arg == "env" for keyword in node.keywords):
                missing_env_lines.append(node.lineno)

        self.assertEqual(missing_env_lines, [])

    def test_agent_wires_usage_artifact_to_bridge_and_metadata(self):
        source = (
            Path(__file__).parents[1]
            / "src"
            / "benchmark_deepswe"
            / "daat_locus_agent.py"
        ).read_text(encoding="utf-8")

        self.assertIn('"--usage-file"', source)
        self.assertIn('"daat-locus-usage.json"', source)
        self.assertIn('"usage_path"', source)


if __name__ == "__main__":
    unittest.main()
