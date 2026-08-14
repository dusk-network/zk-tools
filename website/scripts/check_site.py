from __future__ import annotations

from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]


class PageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.references: list[str] = []
        self.has_title = False
        self.has_description = False
        self.text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if tag == "title":
            self.has_title = True
        if tag == "meta" and values.get("name") == "description":
            self.has_description = bool(values.get("content"))
        if tag in {"a", "link"} and values.get("href"):
            self.references.append(values["href"])
        if tag in {"script", "img"} and values.get("src"):
            self.references.append(values["src"])
        if tag == "img" and values.get("alt"):
            self.text.append(values["alt"])

    def handle_data(self, data: str) -> None:
        self.text.append(data)


def local_target(page: Path, reference: str) -> Path | None:
    parsed = urlsplit(reference)
    if parsed.scheme or parsed.netloc or reference.startswith(("#", "mailto:")):
        return None
    if parsed.path.startswith("/zk-tools/"):
        target = ROOT / parsed.path.removeprefix("/zk-tools/")
    else:
        target = page.parent / parsed.path
    if not parsed.path:
        target = page
    if target.is_dir():
        target /= "index.html"
    return target.resolve()


errors: list[str] = []
pages = sorted(ROOT.rglob("*.html"))

for page in pages:
    parser = PageParser()
    parser.feed(page.read_text(encoding="utf-8"))
    relative = page.relative_to(ROOT)
    if not parser.has_title:
        errors.append(f"{relative}: missing title")
    if not parser.has_description:
        errors.append(f"{relative}: missing meta description")
    plain_text = " ".join(" ".join(parser.text).split())
    if "Powered by Dusk" not in plain_text:
        errors.append(f"{relative}: missing footer attribution")
    for reference in parser.references:
        target = local_target(page, reference)
        if target is not None and not target.exists():
            errors.append(f"{relative}: broken local reference {reference}")

if not pages:
    errors.append("no HTML pages found")

if errors:
    raise SystemExit("\n".join(errors))

print(f"Validated {len(pages)} pages and their local references.")
