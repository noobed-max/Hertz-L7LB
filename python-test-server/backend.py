from http.server import BaseHTTPRequestHandler, HTTPServer

class SimpleHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        # Log the request headers to the console (optional)
        print(f"\n[Backend] Received request for: {self.path}")
        print(f"[Backend] Headers:\n{self.headers}")

        # Send response status code
        self.send_response(200)

        # Send headers
        self.send_header('Content-type', 'text/plain')
        self.end_headers()

        # Send message back to the load balancer
        message = "Hello from Python Backend (Port 8081)!"
        self.wfile.write(bytes(message, "utf8"))

def run(server_class=HTTPServer, handler_class=SimpleHandler, port=8080):
    server_address = ('127.0.0.1', port)
    httpd = server_class(server_address, handler_class)
    print(f"Starting backend server on http://127.0.0.1:{port}...")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass
    httpd.server_close()
    print("Server stopped.")

if __name__ == '__main__':
    run()