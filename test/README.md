# Isolated diagnostic workspace

UI audit evidence is under [runs/ui-audit-2026-11-05](runs/ui-audit-2026-11-05/README.md). Existing permanent test sources remain in UI/tests and tests/. New generated diagnostic assets are kept here where possible.

build/ and run caches are disposable outputs, not release artifacts; test/.gitignore excludes only those generated paths. Keep reports and screenshots necessary to reproduce findings. The root repository ignore rules were not changed.
