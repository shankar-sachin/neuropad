package main

import "encoding/json"

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
