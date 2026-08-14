# ZK Tools website

The documentation site is plain HTML, CSS, and JavaScript so it can be served
directly by GitHub Pages without a package manager or build step.

Preview it locally from the repository root:

```sh
python3 -m http.server 8000 --directory website
```

Then open <http://localhost:8000/>. GitHub Actions publishes the complete
`website/` directory whenever a website file changes on `main`.
