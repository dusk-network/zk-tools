const menuButton = document.querySelector("[data-menu-button]");
const navigation = document.querySelector("[data-navigation]");

if (menuButton && navigation) {
  const setMenuOpen = (open) => {
    menuButton.setAttribute("aria-expanded", String(open));
    menuButton.setAttribute("aria-label", open ? "Close navigation" : "Open navigation");
    navigation.toggleAttribute("data-open", open);
  };

  menuButton.addEventListener("click", () => {
    const open = menuButton.getAttribute("aria-expanded") === "true";
    setMenuOpen(!open);
  });

  navigation.addEventListener("click", (event) => {
    if (event.target.closest("a")) setMenuOpen(false);
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && menuButton.getAttribute("aria-expanded") === "true") {
      setMenuOpen(false);
      menuButton.focus();
    }
  });
}

document.querySelectorAll("[data-copy]").forEach((button) => {
  button.addEventListener("click", async () => {
    const code = button.closest(".code-block")?.querySelector("code");
    if (!code) return;

    await navigator.clipboard.writeText(code.innerText);
    const original = button.textContent;
    button.textContent = "Copied";
    button.dataset.copied = "true";
    window.setTimeout(() => {
      button.textContent = original;
      delete button.dataset.copied;
    }, 1600);
  });
});

document.querySelectorAll("[data-tabs]").forEach((group) => {
  const tabs = [...group.querySelectorAll("[role='tab']")];
  const panels = [...group.querySelectorAll("[role='tabpanel']")];

  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => {
      tabs.forEach((item) => item.setAttribute("aria-selected", "false"));
      panels.forEach((panel) => panel.setAttribute("hidden", ""));
      tab.setAttribute("aria-selected", "true");
      panels[index].removeAttribute("hidden");
    });
  });
});

const year = document.querySelector("[data-year]");
if (year) year.textContent = new Date().getFullYear();
