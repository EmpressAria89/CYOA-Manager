#!/usr/bin/env python3
"""Isolated Linux WebKit check using a private, valid DBZ export; no live data writes.
Requires python-gobject, GTK3/WebKitGTK 4.1 and an exported patched bundle.
"""
import argparse
import http.server
import json
from pathlib import Path
import tempfile
import threading
from urllib.parse import urlparse
import gi

gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2, GLib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--project', type=Path, required=True, help='Valid local DBZ Requiem project export (read only)')
parser.add_argument('--bundle', type=Path, required=True, help='Patched ICC2 Plus bundle exported by the Rust protocol test')
parser.add_argument('--report', type=Path, help='Private output report; keep outside source')
args = parser.parse_args()
root = Path(__file__).resolve().parents[1]
project = json.loads(args.project.read_bytes())
project['rows'] = [row for row in project['rows'] if row['id'] in ['j7tg', 'ax32', 'dyuu']]
assert len(project['rows']) == 3, 'Fixture needs DBZ navigation, Race and Dragon-Kin Type rows'
for row in project['rows']:
    if row['id'] == 'j7tg':
        row['objects'] = [choice for choice in row['objects'] if choice['id'] == 'rcac']
    if row['id'] == 'ax32':
        row['objects'] = [choice for choice in row['objects'] if choice['id'] == 'sqsw']
    if row['id'] == 'dyuu':
        row['objects'] = row['objects'][:1]
child = next(row for row in project['rows'] if row['id'] == 'dyuu')['objects'][0]['id']
for point in project['pointTypes']:
    point['startingSum'] = 99999
# Strip fixture artwork, preserving the real schema and authored styles/requirements.
def strip_images(value):
    if isinstance(value, dict):
        return {key: strip_images(item) for key, item in value.items()}
    if isinstance(value, list):
        return [strip_images(item) for item in value]
    return '' if isinstance(value, str) and value.startswith('data:image/') else value
project_bytes = json.dumps(strip_images(project)).encode()
settings = {'parts': {}}

class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_args):
        pass

    def do_POST(self):
        change = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        settings.update({key: change[key] for key in ['hiddenRows', 'autoHideSections', 'parts', 'scale'] if key in change})
        self.do_GET()

    def do_GET(self):
        path = urlparse(self.path).path
        mime = 'application/javascript' if path.endswith('.js') else 'text/css' if path.endswith('.css') else 'text/html'
        if path == '/project.json':
            data = project_bytes
            mime = 'application/json'
        elif path == '/__manager_reader_settings':
            data = json.dumps({'global': {'parts': {}}, 'local': settings, 'effective': settings}).encode()
            mime = 'application/json'
        elif path == '/__manager_reader_fonts':
            data, mime = b'[]', 'application/json'
        elif path == '/__manager_session':
            data, mime = b'{"theme":"mocha"}', 'application/json'
        elif path in ['/__manager_reader.js', '/__manager_images.js', '/tools.js']:
            name = {'/__manager_reader.js': 'viewer_presentation.js', '/__manager_images.js': 'viewer_images.js', '/tools.js': 'viewer_tools.js'}[path]
            data = (root / 'src-tauri/src' / name).read_bytes()
        elif path == '/js/app.B6d7tc9y.js':
            data = args.bundle.read_bytes()
        elif path == '/':
            data = (root / 'public/viewers/ICC2 Plus/index.html').read_bytes().replace(b'<head>', b'<head><script>window.errors=[];window.addEventListener("error",e=>errors.push(e.message));console.error=(...a)=>errors.push(a.join(" "));</script><script src="/__manager_reader.js"></script><script src="/__manager_images.js"></script>').replace(b'</body>', b'<script src="/tools.js"></script></body>')
        else:
            base = (root / 'public/viewers/ICC2 Plus').resolve()
            file = (base / path.lstrip('/')).resolve()
            if not file.is_relative_to(base) or not file.is_file():
                self.send_error(404)
                return
            data = file.read_bytes()
        self.send_response(200)
        self.send_header('Content-Type', mime)
        self.end_headers()
        self.wfile.write(data)

server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
context = WebKit2.WebContext.new_ephemeral()
context.set_cache_model(WebKit2.CacheModel.DOCUMENT_VIEWER)
view = WebKit2.WebView.new_with_context(context)
window = Gtk.Window()
window.set_default_size(1000, 800)
window.add(view)
manager = view.get_user_content_manager()
manager.register_script_message_handler('report')
results = []

def reported(_manager, message):
    results.append(json.loads(message.get_js_value().to_string()))
    Gtk.main_quit()

manager.connect('script-message-received::report', reported)
script = r'''(async()=>{
 const wait=ms=>new Promise(r=>setTimeout(r,ms));for(let i=0;i<150&&!window.__cyoaManagerNative?.ready();i++)await wait(100);await wait(1500);
 const a=window.__cyoaManagerNative,assert=(ok,msg)=>{if(!ok)throw Error(msg)};
 const shown=()=>!!document.querySelector('.row-dyuu-header')?.getClientRects().length;
 const load=async code=>{a.load(code);await wait(250)};
 const child=CHILD;
 await load('rcac,sqsw,'+child);assert(shown(),'Child visible with parent open');
 await load('sqsw,'+child);assert(!shown(),'Child hidden with parent closed');assert(a.capture().includes('sqsw'),'Race selection preserved');assert(a.capture().includes(child),'Child selection preserved');
 a.setRevealHidden(true);await wait(150);assert(!shown(),'Reveal cheat must keep closed parent tabs closed');a.setRevealHidden(false);
 await load('rcac,sqsw,'+child);assert(shown(),'Reopening parent restores child');const code=a.capture();
 for(let i=0;i<3;i++){
  await fetch('/__manager_reader_settings',{method:'POST',body:JSON.stringify({hiddenRows:['dyuu']})});await __cyoaManagerPresentation.reload();await wait(100);assert(!shown(),'Manual hide');assert(a.capture()===code,'Manual hide changed build');
  await fetch('/__manager_reader_settings',{method:'POST',body:JSON.stringify({hiddenRows:[]})});await __cyoaManagerPresentation.reload();await wait(100);assert(shown(),'Manual restore');assert(a.capture()===code,'Manual restore changed build');
 }
 const title=document.querySelector('[data-manager-card] [data-manager-part=title]');assert(title,'Authored title hook');const beforeSize=parseFloat(getComputedStyle(title).fontSize),beforeWidth=document.querySelector('[data-manager-card]').getBoundingClientRect().width;
 await fetch('/__manager_reader_settings',{method:'POST',body:JSON.stringify({parts:{title:{font:'',size:125}},scale:80})});await __cyoaManagerPresentation.reload();await wait(200);
 assert(Math.abs(parseFloat(getComputedStyle(title).fontSize)-beforeSize*1.25)<.2,'Native live typography');assert(document.querySelector('[data-manager-card]').getBoundingClientRect().width<beforeWidth,'Native density');assert(a.capture()===code,'Display changes altered build');
 const gain=document.querySelector('[data-manager-score-kind=gain]'),cost=document.querySelector('[data-manager-score-kind=cost]');assert(gain&&cost,'Score semantic hooks');assert(getComputedStyle(gain).color!==getComputedStyle(cost).color,'Distinct cost/gain colors');
 await load('sqsw,'+child);await fetch('/__manager_reader_settings',{method:'POST',body:JSON.stringify({autoHideSections:false})});await __cyoaManagerPresentation.reload();await wait(100);assert(shown(),'Automatic hiding opt-out');
 await __cyoaManagerPresentation.open();await wait(200);assert(document.getElementById('manager-quick-tools').shadowRoot.getElementById('manager-display-panel'),'Display panel attached');
 assert(errors.length===0,JSON.stringify(errors));window.webkit.messageHandlers.report.postMessage(JSON.stringify({ok:true,checks:['native rendering','parent close/reopen','selected race and child retained','reveal leaves tabs closed','manual hide/show repeated without build changes','opt-out','display panel','native typography and density preserve builds','cost/gain colors']}));
})().catch(e=>window.webkit.messageHandlers.report.postMessage(JSON.stringify({ok:false,error:e.message,errors:window.errors})));'''.replace('CHILD', json.dumps(child))
started = False

def loaded(current, event):
    global started
    if event == WebKit2.LoadEvent.FINISHED and not started:
        started = True
        current.evaluate_javascript(script, -1, None, None, None, None, None)

view.connect('load-changed', loaded)
GLib.timeout_add_seconds(35, lambda: (results.append({'ok': False, 'error': 'Native check timed out'}), Gtk.main_quit(), False)[-1])
try:
    window.show_all()
    view.load_uri(f'http://127.0.0.1:{server.server_port}/')
    Gtk.main()
finally:
    window.destroy()
    server.shutdown()
result = results[0] if results else {'ok': False, 'error': 'No native result'}
if args.report:
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
raise SystemExit(0 if result.get('ok') else 1)
