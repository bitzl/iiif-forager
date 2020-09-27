# IIIF Forager

:warning: some APIs and configuration parameters are still experimental. Use with caution!

IIIF Presentation API server for images organized in directories, written in Rust.

Try it with samples in this repository:

```sh
$ cargo run config.example.yml
$ curl -s "http://127.0.0.1:8000/sample-123/manifest" | jq
{
    "@context": "http://iiif.io/api/presentation/2/context.json",
    "@id": "http://127.0.0.1:8000/iiif/presentation/sample/manifest",
    "@type": "sc:Manifest",
    ...
```

Features:

- Generate IIIF manifests for images in a directory
- Organize your data in directories and use these as part of an hierarchical id
- Show subdirectories as collections
- Add extra metadata for the manifest in a JSON file _(experimental)_

Planned features:

- Serve Metada embedded in image files as annotations
