package main

import (
	"os"
	"os/exec"
	"path/filepath"
)

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
