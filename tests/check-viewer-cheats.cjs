require('node:fs').mkdirSync('tests/results',{recursive:true});
const {chromium}=require('playwright');
const fs=require('fs'), assert=require('assert');
(async()=>{
const browser=await chromium.launch({headless:true});
for(const [theme,color] of [['mocha','rgb(30, 30, 46)'],['macchiato','rgb(36, 39, 58)'],['frappe','rgb(48, 52, 70)'],['latte','rgb(239, 241, 245)']]){
const p=await browser.newPage({viewport:{width:320,height:600}}); let errors=[];p.on('pageerror',e=>errors.push(e.message));
await p.route('http://cheat.test/**',r=>{
if(r.request().url().endsWith('/__manager_session'))return r.fulfill({json:{theme,cheatsEnabled:true}});
if(r.request().url().endsWith('/__cyoa_manager_viewer_overlay.html'))return r.fulfill({contentType:'text/html',body:fs.readFileSync('src-tauri/src/viewer_overlay.html','utf8')});
return r.fulfill({contentType:'text/html',body:'<style>*{font-family:monospace!important;font-weight:900!important}button,input,select{background:white!important;color:white!important;font-size:40px!important}div{padding:100px!important}</style><body></body>'});});
await p.goto('http://cheat.test/');await p.evaluate(()=>{window.__cyoaManagerNative={setRevealHidden:enabled=>window.__revealed=enabled};window.debugApp={pointTypes:[{name:'Power',startingSum:10}],rows:[{allowedChoices:2,requireds:['a'],objects:[{requireds:['b']}]}]};});
await p.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_overlay.js','utf8')});
await p.getByRole('button',{name:'CYOA Manager tools',exact:true}).click();
await p.getByRole('button',{name:'Show hidden choices',exact:true}).click();assert.equal(await p.evaluate(()=>window.__revealed),true);await p.getByRole('button',{name:'Hide locked choices',exact:true}).click();assert.equal(await p.evaluate(()=>window.__revealed),false);
const panel=p.locator('#cyoa-manager-viewer-panel');
assert.equal(await panel.evaluate(e=>getComputedStyle(e).backgroundColor),color);
assert.equal(await panel.evaluate(e=>getComputedStyle(e).fontWeight),'400');
assert.equal(await p.locator('#cyoa-manager-point-type-select').evaluate(e=>getComputedStyle(e).appearance),'none');
assert.equal(await p.locator('#cyoa-manager-remove-reqs').evaluate(e=>getComputedStyle(e).fontWeight),'400');
await p.locator('#cyoa-manager-point-type-value').fill('27');await p.locator('#cyoa-manager-point-type-value').press('Enter');
await p.getByRole('button',{name:'Remove all requirements'}).click();await p.getByRole('button',{name:'Unlimited Allowed Choices'}).click();
assert.deepEqual(await p.evaluate(()=>({sum:debugApp.pointTypes[0].startingSum,allowed:debugApp.rows[0].allowedChoices,row:debugApp.rows[0].requireds,object:debugApp.rows[0].objects[0].requireds})),{sum:27,allowed:0,row:undefined,object:undefined});
assert.deepEqual(errors,[]);await p.close();
}
await browser.close();console.log('Cheat overlay: four palettes, hostile author CSS isolation, normal font weights, themed select, existing cheat controls passed.');
})().catch(e=>{console.error(e);process.exit(1)});
