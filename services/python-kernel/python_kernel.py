#!/usr/bin/env python3
import json
import sys
import traceback


def write_result(req_id: str, result):
    payload = {"result": result}
    if req_id:
        payload["id"] = req_id
    print(json.dumps(payload), flush=True)


def write_error(req_id: str, code: str, message: str):
    payload = {"error": {"code": code, "message": message}}
    if req_id:
        payload["id"] = req_id
    print(json.dumps(payload), flush=True)


class CaptureStdout:
    def __init__(self):
        self.buffer = []

    def write(self, text):
        self.buffer.append(text)

    def flush(self):
        pass

    def text(self):
        return "".join(self.buffer)


def execute_python(code: str):
    scope = {}
    capture = CaptureStdout()
    real_stdout = sys.stdout
    try:
        sys.stdout = capture
        exec(compile(code, "<neuropad>", "exec"), scope, scope)
    finally:
        sys.stdout = real_stdout
    text = capture.text()
    if not text:
        text = "ok"
    return {"text/plain": text}


for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except json.JSONDecodeError as exc:
        write_error("", "parse_error", str(exc))
        continue

    req_id = req.get("id", "")
    method = req.get("method", "")
    params = req.get("params", {}) or {}

    if method == "execute":
        code = str(params.get("code", ""))
        try:
            write_result(req_id, execute_python(code))
        except Exception as exc:  # noqa: BLE001
            detail = "".join(traceback.format_exception_only(type(exc), exc)).strip()
            write_error(req_id, "execution_error", detail)
    elif method in ("interrupt", "restart", "ping"):
        write_result(req_id, {"ok": True})
    else:
        write_error(req_id, "unknown_method", method)
