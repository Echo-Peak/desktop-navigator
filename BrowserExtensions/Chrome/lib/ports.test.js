import test from "node:test";
import assert from "node:assert/strict";
import { retryPorts, wsUrl, DEFAULT_PORT, MAX_EXTRA_PORTS } from "./ports.js";

test("retryPorts covers 35282 through 35292 inclusive", () => {
  const ports = retryPorts();
  assert.equal(ports.length, MAX_EXTRA_PORTS + 1);
  assert.equal(ports[0], DEFAULT_PORT);
  assert.equal(ports[ports.length - 1], 35292);
});

test("retryPorts is contiguous", () => {
  const ports = retryPorts();
  for (let i = 1; i < ports.length; i += 1) {
    assert.equal(ports[i] - ports[i - 1], 1);
  }
});

test("wsUrl builds a ws url", () => {
  assert.equal(wsUrl("localhost", 35282), "ws://localhost:35282");
});
