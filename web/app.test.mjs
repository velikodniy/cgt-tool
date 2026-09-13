import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import vm from "node:vm";

function element() {
  return {
    children: [],
    classList: { add() {}, remove() {} },
    addEventListener() {},
    setAttribute() {},
    append(...children) {
      this.children.push(...children);
    },
    appendChild(child) {
      this.children.push(child);
    },
    set innerHTML(_value) {
      assert.fail("Toast content must not use innerHTML");
    },
  };
}

async function loadApp() {
  const elements = new Map();
  const document = {
    createElement: element,
    getElementById(id) {
      if (!elements.has(id)) elements.set(id, element());
      return elements.get(id);
    },
  };
  const source = await readFile(new URL("app.js", import.meta.url), "utf8");
  const context = vm.createContext({
    document,
    window: {},
    console,
    setTimeout() {},
    init: async () => {},
    calculate_tax() {},
    parse_transactions() {},
    validate_dsl() {},
  });
  vm.runInContext(source.replace(/^import .*;$/gm, ""), context);
  await Promise.resolve();
  document.getElementById("toastContainer").children.length = 0;
  return { context, document };
}

test("parser errors remain text in toast messages", async () => {
  const { context, document } = await loadApp();
  const message = 'Parsing error: <img src=x onerror="document.body.dataset.injected=1"> & text';
  context.showToast(message, "error");

  const [toast] = document.getElementById("toastContainer").children;
  assert.equal(toast.className, "toast error");
  assert.equal(toast.children.length, 2);
  assert.equal(toast.children[0].textContent, "✕");
  assert.equal(toast.children[1].textContent, message);
});
