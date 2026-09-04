# Deferred work

- Settings can be closed while an async save is still pending, so the parent may briefly re-read the previous persisted preferences. Address in a focused settings lifecycle change if users reproduce it.
