package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
)

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
