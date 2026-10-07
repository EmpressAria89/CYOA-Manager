// Serve the built frontend and run reproducible browser checks with mocked backend calls.
const {spawn,spawnSync}=require('node:child_process'),http=require('node:http');
const port=5797,url=`http://127.0.0.1:${port}`;
const server=spawn(process.execPath,['node_modules/vite/bin/vite.js','preview','--host','127.0.0.1','--port',String(port),'--strictPort'],{stdio:'ignore'});
let failure=null;server.on('error',e=>failure=e);server.on('exit',code=>{if(code!==null)failure=new Error(`Preview exited (${code})`)});
const ready=()=>new Promise(resolve=>{http.get(url,r=>{r.resume();resolve(r.statusCode===200)}).on('error',()=>resolve(false));});
(async()=>{try{
 for(let i=0;i<100;i++){if(failure)throw failure;if(await ready())break;if(i===99)throw new Error('Preview failed to start');await new Promise(r=>setTimeout(r,100));}
 for(const file of ['smoke-ui.cjs','check-ui-revision.cjs','check-author-aliases.cjs','check-viewer-dock.cjs','check-viewer-cheats.cjs']){
  const result=spawnSync(process.execPath,[`tests/${file}`],{stdio:'inherit',env:{...process.env,CYOA_TEST_URL:url}});if(result.status!==0)throw new Error(`${file} failed (${result.status})`);
 }
}finally{server.kill('SIGTERM');}})().catch(e=>{console.error(e.message);process.exitCode=1});
