#!/usr/bin/env python3
"""Simple HTTP server for serving the markdown-academic documentation."""

import http.server
import socketserver
import os
import sys
import webbrowser
from functools import partial

PORT = 8000
DOCS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "docs")


def main():
    port = PORT
    if len(sys.argv) > 1:
        try:
            port = int(sys.argv[1])
        except ValueError:
            print(f"Invalid port: {sys.argv[1]}")
            sys.exit(1)

    os.chdir(DOCS_DIR)
    
    handler = partial(http.server.SimpleHTTPRequestHandler, directory=DOCS_DIR)
    
    with socketserver.TCPServer(("", port), handler) as httpd:
        url = f"http://localhost:{port}"
        print(f"Serving docs at {url}")
        print(f"Directory: {DOCS_DIR}")
        print("Press Ctrl+C to stop\n")
        
        # Open browser automatically
        webbrowser.open(url)
        
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nShutting down...")
            sys.exit(0)


if __name__ == "__main__":
    main()
