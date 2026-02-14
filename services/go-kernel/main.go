package main

import (
	"bufio"
	"encoding/json"
	"io"
	"os"
	"strings"
)

func main() {
	reader := bufio.NewReader(os.Stdin)
	writer := bufio.NewWriter(os.Stdout)
	defer writer.Flush()

	for {
		line, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				return
			}
			writeErr(writer, "", "io_error", err.Error())
			continue
		}
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}

		var req request
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			writeErr(writer, "", "parse_error", err.Error())
			continue
		}

		switch req.Method {
		case "execute":
			var p executeParams
			if err := json.Unmarshal(req.Params, &p); err != nil {
				writeErr(writer, req.ID, "bad_request", err.Error())
				continue
			}
			out, execErr := runGoCode(p.Code)
			if execErr != nil {
				writeErr(writer, req.ID, "execution_error", execErr.Error())
				continue
			}
			writeResult(writer, req.ID, map[string]string{"text/plain": out})
		case "interrupt", "restart", "ping":
			writeResult(writer, req.ID, map[string]bool{"ok": true})
		default:
			writeErr(writer, req.ID, "unknown_method", req.Method)
		}
	}
}
