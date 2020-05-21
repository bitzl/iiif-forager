# IIIF Forager

IIIF Presentation API server for images organized in directories, written in Rust.

Try it with samples in this repository:

```sh
$ cargo run
$ curl -s "http://127.0.0.1:8000/sample/manifest" | jq
{
    "@context": "http://iiif.io/api/presentation/2/context.json",
    "@id": "http://127.0.0.1:8000/iiif/presentation/sample/manifest",
    "@type": "sc:Manifest",
    ...
```                                                 