#!/usr/bin/env ruby
require "json"
require "securerandom"

$stdout.sync = true

def write_result(id, result)
  payload = {}
  payload["id"] = id unless id.nil? || id.empty?
  payload["result"] = result
  puts(JSON.generate(payload))
end

def write_error(id, code, message)
  payload = {}
  payload["id"] = id unless id.nil? || id.empty?
  payload["error"] = { "code" => code, "message" => message }
  puts(JSON.generate(payload))
end

while (line = STDIN.gets)
  line = line.strip
  next if line.empty?

  begin
    req = JSON.parse(line)
  rescue JSON::ParserError => e
    write_error("", "parse_error", e.message)
    next
  end

  id = req["id"] || SecureRandom.uuid
  method = req["method"]
  params = req["params"] || {}

  case method
  when "execute"
    begin
      code = params["code"].to_s
      output = eval(code).inspect
      write_result(id, { "text/plain" => output })
    rescue StandardError => e
      write_error(id, "execution_error", e.message)
    end
  when "interrupt", "restart", "ping"
    write_result(id, { "ok" => true })
  else
    write_error(id, "unknown_method", method.to_s)
  end
end
