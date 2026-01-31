import http.server
import socketserver
import sys
import os

class MyHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

def run_server(path, ip, port):
    try:
        os.chdir(path)
    except FileNotFoundError:
        print(f"Error: Directory '{path}' not found.")
        return

    handler = MyHandler
    handler.extensions_map.update({
        '.wasm': 'application/wasm',
    })

    socketserver.TCPServer.allow_reuse_address = True

    try:
        with socketserver.TCPServer((ip, port), handler) as httpd:
            print(f"Serving '{path}' at http://{ip}:{port}")
            httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped.")
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    dir_to_serve = sys.argv[1] if len(sys.argv) > 1 else "."
    target_ip = sys.argv[2] if len(sys.argv) > 2 else "0.0.0.0"
    target_port = int(sys.argv[3]) if len(sys.argv) > 3 else 8000

    run_server(dir_to_serve, target_ip, target_port)