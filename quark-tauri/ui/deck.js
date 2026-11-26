"use strict";
// Mode Deck : activation, focus spatial, navigation manette et clavier.

const QuarkDeck = (() => {
  const state = {
    active: false,
    ready: false,
    mode: "auto",
    status: { steam_deck: false, gamescope: false, gamepad: false },
    previousFullscreen: false,
    seen: false,
  };
  const dirs = new Set(["up", "down", "left", "right"]);

  const modalOpen = () => !!$("#modal .modal");
  const scope = () => (modalOpen() ? $("#modal") : $("#shell"));

  function visible(el) {
    if (!el || el.disabled || el.hidden || el.getAttribute("aria-hidden") === "true") return false;
    if (el.closest("[hidden]")) return false;
    const style = getComputedStyle(el);
    if (style.display === "none" || style.visibility === "hidden" || style.opacity === "0") return false;
    const rect = el.getBoundingClientRect();
    return rect.width > 1 && rect.height > 1;
  }

  function focusables() {
    const root = scope();
    if (!root) return [];
    return $$("button, a[href], input, select, textarea, [tabindex]", root).filter((el) => {
      if (el.tabIndex < 0) return false;
      if (el.type === "hidden") return false;
      return visible(el);
    });
  }

  function mark(el) {
    $$(".deck-focus").forEach((node) => node.classList.remove("deck-focus"));
    if (el) el.classList.add("deck-focus");
  }

  function ensureFocus(forceFirst = false) {
    if (!state.active) return null;
    const list = focusables();
    if (!list.length) return null;
    let current = document.activeElement;
    if (!forceFirst && list.includes(current)) return current;
    current = list[0];
    current.focus({ preventScroll: true });
    current.scrollIntoView({ block: "nearest", inline: "nearest" });
    mark(current);
    return current;
  }

  function move(direction) {
    const list = focusables();
    if (!list.length) return;
    let current = document.activeElement;
    if (!list.includes(current)) {
      ensureFocus(true);
      return;
    }
    const base = current.getBoundingClientRect();
    const bx = base.left + base.width / 2;
    const by = base.top + base.height / 2;
    let best = null;
    let bestScore = Infinity;
    for (const el of list) {
      if (el === current) continue;
      const rect = el.getBoundingClientRect();
      const x = rect.left + rect.width / 2;
      const y = rect.top + rect.height / 2;
      const dx = x - bx;
      const dy = y - by;
      const primary =
        direction === "up" ? -dy : direction === "down" ? dy : direction === "left" ? -dx : dx;
      if (primary < 8) continue;
      const secondary =
        direction === "up" || direction === "down" ? Math.abs(dx) : Math.abs(dy);
      const overlap =
        direction === "up" || direction === "down"
          ? Math.max(0, Math.min(base.right, rect.right) - Math.max(base.left, rect.left))
          : Math.max(0, Math.min(base.bottom, rect.bottom) - Math.max(base.top, rect.top));
      const score = primary + secondary * 1.45 - overlap * 0.28;
      if (score < bestScore) {
        bestScore = score;
        best = el;
      }
    }
    if (best) {
      best.focus({ preventScroll: true });
      best.scrollIntoView({ block: "nearest", inline: "nearest" });
      mark(best);
    }
  }

  function activate(el) {
    if (!el) return;
    if (el.matches("input, textarea, select")) {
      el.focus();
      if (el.tagName === "SELECT") el.click();
      return;
    }
    el.click();
    setTimeout(() => refocus(), 0);
  }

  function back() {
    if (modalOpen()) {
      closeModal();
      refocus();
      return;
    }
    if (app.view === "library" && app.open) {
      app.open = false;
      renderSheet();
      refocus();
      return;
    }
    if (app.view !== "library") {
      app.view = "library";
      app.open = false;
      renderAll();
      refocus();
    }
  }

  function menu() {
    const items = [
      ["home", t("x.home"), "home"],
      ["library", t("nav.library"), "grid"],
      ["settings", t("settings.title"), "gear"],
      ["exit", t("deck.exit"), "back"],
      ["quit", t("deck.quit"), "x"],
    ];
    modal(`<h2>${esc(t("deck.menu"))}</h2><div class="deck-menu">${items
      .map(
        ([action, label, icon]) =>
          `<button class="btn sec" data-deck-menu="${action}">${ic(icon)}<span>${esc(label)}</span></button>`,
      )
      .join("")}</div>`);
    $$("#modal [data-deck-menu]").forEach((button) => {
      button.onclick = () => {
        const action = button.dataset.deckMenu;
        closeModal();
        if (action === "exit") return disable();
        if (action === "quit") return invoke("win_close");
        if (action === "settings") app.stab = "appearance";
        app.view = action === "home" || action === "library" ? action : "settings";
        app.open = false;
        renderAll();
        refocus();
      };
    });
    refocus();
  }

  function hints() {
    let el = $("#deck-hints");
    if (!el) {
      el = document.createElement("div");
      el.id = "deck-hints";
      el.className = "deck-hints";
      el.innerHTML = [
        `<span><b>A</b>${esc(t("deck.hint.open"))}</span>`,
        `<span><b>B</b>${esc(t("deck.hint.back"))}</span>`,
        `<span><b>X</b>${esc(t("deck.hint.play"))}</span>`,
        `<span><b>Y</b>${esc(t("deck.hint.favorite"))}</span>`,
        `<span><b>LB/RB</b>${esc(t("deck.hint.tabs"))}</span>`,
        `<span><b>Start</b>${esc(t("deck.hint.menu"))}</span>`,
      ].join("");
      document.body.append(el);
    }
    el.hidden = false;
  }

  function enable() {
    if (state.active) return;
    state.active = true;
    document.body.classList.add("deck");
    hints();
    invoke("win_set_fullscreen", { active: true }).catch(() => {});
    refocus();
  }

  function disable() {
    if (!state.active) return;
    state.active = false;
    document.body.classList.remove("deck");
    const el = $("#deck-hints");
    if (el) el.hidden = true;
    mark(null);
    invoke("win_set_fullscreen", { active: state.previousFullscreen }).catch(() => {});
    refocus();
  }

  function toggle() {
    state.active ? disable() : enable();
  }

  function applyMode() {
    if (!state.ready) return;
    if (state.mode === "on") enable();
    else if (state.mode === "off") disable();
    else if (state.status.steam_deck || state.status.gamescope || app.data?.deck) enable();
  }

  function handle(action, pressed) {
    if (!pressed) return;
    if (!state.active) {
      if (state.mode === "auto" && !app.running.size) enable();
      return;
    }
    if (dirs.has(action)) return move(action);
    if (action === "confirm") return activate(document.activeElement);
    if (action === "back") return back();
    if (action === "menu") return menu();
    if (action === "play") return acts.play();
    if (action === "favorite") return acts.fav();
    if (action === "tab_prev" || action === "tab_next") {
      const tabs = $$("#top-tabs [data-view]");
      if (!tabs.length) return;
      const current = tabs.findIndex((tab) => tab.dataset.view === app.view);
      const delta = action === "tab_next" ? 1 : -1;
      const next = tabs[(Math.max(0, current) + delta + tabs.length) % tabs.length];
      if (next) {
        setView(next.dataset.view);
        refocus();
      }
    }
  }

  function handleKey(event) {
    if (!state.active) return false;
    if (/INPUT|TEXTAREA|SELECT/.test(document.activeElement?.tagName || "")) {
      if (event.key === "Escape") {
        document.activeElement.blur();
        refocus();
        return true;
      }
      return false;
    }
    const map = {
      ArrowUp: "up",
      ArrowDown: "down",
      ArrowLeft: "left",
      ArrowRight: "right",
      Enter: "confirm",
      Escape: "back",
      Home: "menu",
    };
    const action = map[event.key];
    if (!action) return false;
    event.preventDefault();
    handle(action, true);
    return true;
  }

  function refocus() {
    if (!state.active) return;
    requestAnimationFrame(() => ensureFocus(false));
  }

  async function init() {
    state.mode = app.cfg.deck_mode || "auto";
    state.previousFullscreen = false;
    try {
      state.status = await invoke("deck_status");
    } catch {
      state.status = { steam_deck: false, gamescope: false, gamepad: false };
    }
    state.ready = true;
    applyMode();
    const observer = new MutationObserver(() => refocus());
    observer.observe(document.body, { childList: true, subtree: true });
    document.addEventListener("focusin", () => {
      if (state.active) mark(document.activeElement);
    });
    document.addEventListener("keydown", (event) => {
      if (handleKey(event)) event.stopImmediatePropagation();
    }, true);
    renderTop();
  }

  return {
    init,
    handle,
    handleKey,
    enable,
    disable,
    toggle,
    applyMode,
    refocus,
    available: () => state.active || state.status.gamepad || state.status.steam_deck || state.status.gamescope,
    active: () => state.active,
    mode: (value) => {
      state.mode = value;
      applyMode();
    },
  };
})();
