import unittest
import tempfile
from pathlib import Path

from benchmark_deepswe.bridge import (
    COMMIT_INSTRUCTION,
    build_instruction,
    ensure_daemon_port,
    extract_status_summary_token_usage,
    parse_model_retry_events,
    prepare_visible_home_logs,
    summarize_token_usage,
)


class BridgeTests(unittest.TestCase):
    def test_adds_daemon_section_when_missing(self):
        self.assertEqual(
            ensure_daemon_port('main_model = "gpt-5.5"\n', 53826),
            'main_model = "gpt-5.5"\n\n[daemon]\nport = 53826\n',
        )

    def test_replaces_existing_daemon_port(self):
        self.assertEqual(
            ensure_daemon_port("[daemon]\nport = 53825\n\n[models.default]\n", 60001),
            "[daemon]\nport = 60001\n\n[models.default]\n",
        )

    def test_inserts_daemon_port_into_existing_section(self):
        self.assertEqual(
            ensure_daemon_port("[daemon]\n\n[models.default]\n", 60001),
            "[daemon]\nport = 60001\n\n[models.default]\n",
        )

    def test_build_instruction_appends_commit_requirement_once(self):
        once = build_instruction("Fix the bug.")
        twice = build_instruction(once)
        self.assertIn(COMMIT_INSTRUCTION, once)
        self.assertEqual(once, twice)

    def test_prepare_visible_home_logs_creates_agent_visible_log_dir(self):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            home = root / "home"
            agent_logs = root / "agent"

            mirror = prepare_visible_home_logs(home, agent_logs)

            self.assertEqual(mirror, agent_logs / "daat-locus-home-logs")
            self.assertTrue(mirror.is_dir())
            self.assertTrue((home / "logs").exists())
            if (home / "logs").is_symlink():
                (home / "logs" / "daat-locus.log").write_text(
                    "started\n",
                    encoding="utf-8",
                )
                self.assertEqual(
                    (mirror / "daat-locus.log").read_text(encoding="utf-8"),
                    "started\n",
                )

    def test_summarize_token_usage_combines_main_and_judge_totals(self):
        summary = summarize_token_usage(
            {
                "main_model": "gpt-5.5",
                "judge_model": "judge",
                "efficient_model": "small",
                "main": {
                    "total_token_usage": {
                        "input_tokens": 10,
                        "cached_input_tokens": 4,
                        "output_tokens": 3,
                        "reasoning_output_tokens": 2,
                        "total_tokens": 15,
                    }
                },
                "judge": {
                    "total_token_usage": {
                        "input_tokens": 7,
                        "cached_input_tokens": 1,
                        "output_tokens": 5,
                        "reasoning_output_tokens": 0,
                        "total_tokens": 12,
                    }
                },
            }
        )

        self.assertEqual(summary["main_model"], "gpt-5.5")
        self.assertEqual(summary["combined"]["input_tokens"], 17)
        self.assertEqual(summary["combined"]["cached_input_tokens"], 5)
        self.assertEqual(summary["combined"]["output_tokens"], 8)
        self.assertEqual(summary["combined"]["reasoning_output_tokens"], 2)
        self.assertEqual(summary["combined"]["total_tokens"], 27)

    def test_extract_status_summary_token_usage_selects_session(self):
        usage = {"main": {"total_token_usage": {"total_tokens": 5}}}

        self.assertIs(
            extract_status_summary_token_usage(
                {
                    "ok": True,
                    "value": {
                        "sessions": [
                            {
                                "session": {"session_id": "other"},
                                "dashboard": {"token_usage": {}},
                            },
                            {
                                "session": {"session_id": "target"},
                                "dashboard": {"token_usage": usage},
                            },
                        ]
                    },
                },
                "target",
            ),
            usage,
        )

    def test_parse_model_retry_events_preserves_multiline_error(self):
        events = parse_model_retry_events(
            "2026-06-23T03:40:14.487298Z  WARN ThreadId(01) "
            "daat_locus::runtime::runtime_loop::model_driver: "
            "run_agent_turn retry #1 after 300ms "
            "(model=gpt-5.5, messages=152, tools=29, estimated_input_tokens=63950): "
            "Codex Responses stream read failed: streaming response body read failed\n"
            "url=https://chatgpt.com/backend-api/codex/responses\n"
            "kind=stream_body_read\n"
            "2026-06-23T03:40:15.000000Z  INFO next log\n"
        )

        self.assertEqual(len(events), 1)
        self.assertEqual(events[0]["attempt"], 1)
        self.assertEqual(events[0]["backoff_ms"], 300)
        self.assertEqual(events[0]["model"], "gpt-5.5")
        self.assertEqual(events[0]["messages"], 152)
        self.assertEqual(events[0]["tools"], 29)
        self.assertEqual(events[0]["estimated_input_tokens"], 63950)
        self.assertIn("kind=stream_body_read", events[0]["error"])


if __name__ == "__main__":
    unittest.main()
