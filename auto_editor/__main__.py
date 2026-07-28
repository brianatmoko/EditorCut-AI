"""Allows `python -m auto_editor` as CLI entry point."""

from auto_editor.main import main

if __name__ == "__main__":
    import sys
    sys.exit(main())
