package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

type request struct {
	ID     string          `json:"id"`
	Method string          `json:"method"`
	Params json.RawMessage `json:"params"`
}

type envelope struct {
	ID     *string     `json:"id,omitempty"`
	Event  *string     `json:"event,omitempty"`
	Result interface{} `json:"result,omitempty"`
	Error  *ipcError   `json:"error,omitempty"`
}

type ipcError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
}

type executeParams struct {
	Code string `json:"code"`
}

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

func runGoCode(code string) (string, error) {
	tmpDir, err := os.MkdirTemp("", "neuropad-go-*")
	if err != nil {
		return "", err
	}
	defer os.RemoveAll(tmpDir)

	filePath := filepath.Join(tmpDir, "main.go")
	source := "package main\n\nimport \"fmt\"\n\nfunc main(){\n" + code + "\n}\n"
	if err := os.WriteFile(filePath, []byte(source), 0644); err != nil {
		return "", err
	}

	cmd := exec.Command("go", "run", filePath)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return string(output), err
	}
	return string(output), nil
}

func writeErr(writer *bufio.Writer, id string, code string, message string) {
	env := envelope{
		Error: &ipcError{
			Code:    code,
			Message: message,
		},
	}
	if id != "" {
		env.ID = &id
	}
	mustWrite(writer, env)
}

func writeResult(writer *bufio.Writer, id string, result interface{}) {
	env := envelope{
		Result: result,
	}
	if id != "" {
		env.ID = &id
	}
	mustWrite(writer, env)
}

func mustWrite(writer *bufio.Writer, env envelope) {
	payload, err := json.Marshal(env)
	if err != nil {
		fmt.Fprintf(os.Stderr, "marshal response error: %v\n", err)
		return
	}
	writer.Write(payload)
	writer.WriteString("\n")
	writer.Flush()
}
