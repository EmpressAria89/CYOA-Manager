(function () {
  var buttonId = "cyoa-manager-viewer-gear";
  var panelId = "cyoa-manager-viewer-panel";
  var selectId = "cyoa-manager-point-type-select";
  var inputId = "cyoa-manager-point-type-value";
  var removeReqsButtonId = "cyoa-manager-remove-reqs";
  var unlimitedChoicesButtonId = "cyoa-manager-unlimited-choices";
  var scriptPath = "/__cyoa_manager_viewer_overlay.js";
  var templatePath = "/__cyoa_manager_viewer_overlay.html";
  var markupPromise = null;
  var overlayRoot = null;
  var overlayTheme = "mocha";
  var palettes = {
    mocha: ["#1e1e2e", "#313244", "#45475a", "#585b70", "#cdd6f4", "#a6adc8", "#89b4fa", "dark"],
    macchiato: ["#24273a", "#363a4f", "#494d64", "#5b6078", "#cad3f5", "#a5adcb", "#8aadf4", "dark"],
    frappe: ["#303446", "#414559", "#51576d", "#626880", "#c6d0f5", "#a5adce", "#8caaee", "dark"],
    latte: ["#eff1f5", "#ccd0da", "#bcc0cc", "#acb0be", "#4c4f69", "#6c6f85", "#1e66f5", "light"]
  };

  function findControl(id) {
    return overlayRoot ? overlayRoot.getElementById(id) : null;
  }

  function applyTheme(host) {
    var colors = palettes[overlayTheme] || palettes.mocha;
    ["base", "surface", "hover", "border", "text", "subtext", "accent", "scheme"].forEach(function (key, index) {
      host.style.setProperty("--" + key, colors[index]);
    });
  }

  function getSupportedAppState() {
    try {
      var appState =
        window.app
        && window.app.__vue__
        && window.app.__vue__.$store
        && window.app.__vue__.$store.state
        && window.app.__vue__.$store.state.app
        || window.debugApp;

      if (!appState || !Array.isArray(appState.pointTypes)) {
        return null;
      }

      return appState;
    } catch (_error) {
      return null;
    }
  }

  function getPointTypeOptions(appState) {
    return appState.pointTypes
      .map(function (item, index) {
        if (!item) {
          return null;
        }

        var label = getPointTypeLabel(item, index);
        if (!label) {
          return null;
        }

        return {
          index: index,
          item: item,
          label: label,
        };
      })
      .filter(function (entry) {
        return Boolean(entry);
      });
  }

  function getPointTypeLabel(item, index) {
    if (typeof item.name === "string" && item.name.trim() !== "") {
      return item.name.trim();
    }

    return "Point type " + (index + 1);
  }

  function ensureMarkup() {
    if (findControl(buttonId) && findControl(panelId)) {
      return Promise.resolve();
    }

    if (markupPromise) {
      return markupPromise;
    }

    markupPromise = window.fetch(templatePath, { cache: "no-store" })
      .then(function (response) {
        if (!response.ok) {
          throw new Error("overlay template request failed");
        }
        return response.text();
      })
      .then(function (markup) {
        if (!findControl(buttonId) && !findControl(panelId)) {
          var host = document.createElement("div");
          host.id = "cyoa-manager-cheat-overlay";
          // Author CSS and CYOA font overrides cannot cross this boundary.
          overlayRoot = host.attachShadow({ mode: "open" });
          overlayRoot.innerHTML = markup;
          applyTheme(host);
          document.body.appendChild(host);
        }
      })
      .catch(function (_error) {
        markupPromise = null;
      });

    return markupPromise;
  }

  function populatePointTypeSelect(appState) {
    var select = findControl(selectId);
    if (!select) {
      return;
    }

    var options = getPointTypeOptions(appState);
    var previousValue = select.value;
    select.innerHTML = "";

    options.forEach(function (entry) {
      var option = document.createElement("option");
      option.value = String(entry.index);
      option.textContent = entry.label;
      select.appendChild(option);
    });

    if (previousValue && options.some(function (entry) { return String(entry.index) === previousValue; })) {
      select.value = previousValue;
    } else if (options.length > 0) {
      select.value = String(options[0].index);
    }
  }

  function syncSelectedValue(appState) {
    var select = findControl(selectId);
    var input = findControl(inputId);
    if (!select || !input) {
      return;
    }

    var options = getPointTypeOptions(appState);
    var selected = options.find(function (entry) {
      return String(entry.index) === select.value;
    }) || options[0];

    if (!selected) {
      input.value = "";
      input.disabled = true;
      return;
    }

    if (select.value !== String(selected.index)) {
      select.value = String(selected.index);
    }

    input.disabled = false;
    input.value = String(selected.item.startingSum ?? 0);
  }

  function applyStartingSum(appState) {
    var select = findControl(selectId);
    var input = findControl(inputId);
    if (!select || !input) {
      return;
    }

    var options = getPointTypeOptions(appState);
    var selected = options.find(function (entry) {
      return String(entry.index) === select.value;
    });

    if (!selected) {
      return;
    }

    var parsed = Number(input.value);
    if (!Number.isFinite(parsed)) {
      input.value = String(selected.item.startingSum ?? 0);
      return;
    }

    selected.item.startingSum = parsed;
    input.value = String(parsed);
  }

  function removeAllRequirements(appState) {
    if (!appState || !Array.isArray(appState.rows)) {
      return;
    }

    appState.rows.forEach(function (row) {
      if (!row || typeof row !== "object") {
        return;
      }

      if (Object.prototype.hasOwnProperty.call(row, "requireds")) {
        delete row.requireds;
      }

      if (!Array.isArray(row.objects)) {
        return;
      }

      row.objects.forEach(function (item) {
        if (!item || typeof item !== "object") {
          return;
        }

        if (Object.prototype.hasOwnProperty.call(item, "requireds")) {
          delete item.requireds;
        }
      });
    });
  }

  function setUnlimitedAllowedChoices(appState) {
    if (!appState || !Array.isArray(appState.rows)) {
      return;
    }

    appState.rows.forEach(function (row) {
      if (!row || typeof row !== "object") {
        return;
      }

      row.allowedChoices = 0;
    });
  }

  function ensurePanel(appState) {
    var panel = findControl(panelId);
    var select = findControl(selectId);
    var input = findControl(inputId);
    var removeReqsButton = findControl(removeReqsButtonId);
    var unlimitedChoicesButton = findControl(unlimitedChoicesButtonId);
    if (!panel || !select || !input || !removeReqsButton || !unlimitedChoicesButton) {
      return;
    }

    var reveal = findControl("manager-reveal-hidden");
    if(reveal && window.__cyoaManagerNative?.setRevealHidden){
      reveal.hidden=false;
      if(!reveal.dataset.cyoaManagerBound){
        reveal.addEventListener("click",function(){
          var enabled=reveal.getAttribute("aria-pressed")!=="true";
          window.__cyoaManagerNative.setRevealHidden(enabled);
          reveal.setAttribute("aria-pressed",String(enabled));
          reveal.textContent=enabled?"Hide locked choices":"Show hidden choices";
        });
        reveal.dataset.cyoaManagerBound="true";
      }
    }
    populatePointTypeSelect(appState);

    if (!select.dataset.cyoaManagerBound) {
      select.addEventListener("change", function () {
        var liveAppState = getSupportedAppState();
        if (liveAppState) {
          syncSelectedValue(liveAppState);
        }
      });
      select.dataset.cyoaManagerBound = "true";
    }

    if (!input.dataset.cyoaManagerBound) {
      input.addEventListener("change", function () {
        var liveAppState = getSupportedAppState();
        if (liveAppState) {
          applyStartingSum(liveAppState);
        }
      });

      input.addEventListener("keydown", function (event) {
        if (event.key === "Enter") {
          var liveAppState = getSupportedAppState();
          if (liveAppState) {
            applyStartingSum(liveAppState);
          }
        }
      });
      input.dataset.cyoaManagerBound = "true";
    }

    if (!removeReqsButton.dataset.cyoaManagerBound) {
      removeReqsButton.addEventListener("click", function () {
        var liveAppState = getSupportedAppState();
        if (liveAppState) {
          removeAllRequirements(liveAppState);
          removeReqsButton.textContent = "Requirements removed";
          window.setTimeout(function () {
            removeReqsButton.textContent = "Remove all requirements";
          }, 1200);
        }
      });
      removeReqsButton.dataset.cyoaManagerBound = "true";
    }

    if (!unlimitedChoicesButton.dataset.cyoaManagerBound) {
      unlimitedChoicesButton.addEventListener("click", function () {
        var liveAppState = getSupportedAppState();
        if (liveAppState) {
          setUnlimitedAllowedChoices(liveAppState);
          unlimitedChoicesButton.textContent = "Allowed choices unlocked";
          window.setTimeout(function () {
            unlimitedChoicesButton.textContent = "Unlimited Allowed Choices";
          }, 1200);
        }
      });
      unlimitedChoicesButton.dataset.cyoaManagerBound = "true";
    }

    syncSelectedValue(appState);
  }

  function togglePanel() {
    var panel = findControl(panelId);
    if (!panel) {
      return;
    }

    panel.hidden = !panel.hidden;
  }

  function ensureButton(appState) {
    var button = findControl(buttonId);
    if (!button) {
      return;
    }

    if (!button.dataset.cyoaManagerBound) {
      button.addEventListener("click", function () {
        var liveAppState = getSupportedAppState();
        togglePanel();
        if (liveAppState) {
          ensurePanel(liveAppState);
        }
      });
      button.dataset.cyoaManagerBound = "true";
    }

    ensurePanel(appState);
  }

  function schedulePointTypeRefresh() {
    var attempts = 0;
    var timer = window.setInterval(function () {
      attempts += 1;

      var liveAppState = getSupportedAppState();
      if (!liveAppState) {
        if (attempts > 40) {
          window.clearInterval(timer);
        }
        return;
      }

      ensurePanel(liveAppState);

      var select = findControl(selectId);
      if ((select && select.options.length > 0) || attempts > 40) {
        window.clearInterval(timer);
      }
    }, 500);
  }

  function tryAttachOverlay() {
    if (!document.body || !document.head) {
      return false;
    }

    var appState = getSupportedAppState();
    if (!appState) {
      return false;
    }

    ensureMarkup().then(function () {
      var liveAppState = getSupportedAppState();
      if (liveAppState) {
        ensureButton(liveAppState);
        schedulePointTypeRefresh();
      }
    });
    return true;
  }

  async function startPolling() {
    try {
      var session = await (await fetch("/__manager_session")).json();
      if (!session.cheatsEnabled) return;
      overlayTheme = session.theme || "mocha";
    } catch (_error) { return; }
    if (tryAttachOverlay()) {
      return;
    }

    var attempts = 0;
    var timer = window.setInterval(function () {
      attempts += 1;
      if (tryAttachOverlay() || attempts > 120) {
        window.clearInterval(timer);
      }
    }, 500);
  }

  if (window.location.pathname === scriptPath || window.location.pathname === templatePath) {
    return;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", startPolling, { once: true });
  } else {
    startPolling();
  }
})();