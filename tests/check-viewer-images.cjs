// Reader lifecycle regression: offscreen sources must release without losing layout.
const {chromium}=require('playwright'),fs=require('fs'),assert=require('node:assert/strict');
(async()=>{
 const browser=await chromium.launch({headless:true});
 try {
  const page=await browser.newPage({viewport:{width:800,height:600}});
  await page.route('http://images.test/**',route=>route.fulfill({contentType:'image/svg+xml',body:'<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400"><rect width="800" height="400" fill="blue"/></svg>'}));
  await page.route('http://images.test/',route=>route.fulfill({contentType:'text/html',body:'<html></html>'}));
  await page.goto('http://images.test');
  await page.setContent('<style>img{display:block;max-width:100%;height:auto;margin:20px 0}body{margin:0}</style>');
  await page.addScriptTag({content:fs.readFileSync('src-tauri/src/viewer_images.js','utf8')});
  await page.evaluate(()=>{
   const svg='/__manager_images/fixture.svg?w=800&h=400';
   for(let n=0;n<60;n++){const img=document.createElement('img');img.id='img-'+n;document.body.append(img);window.__cyoaManagerImages.setSource(img,'src',svg);}
   // Authored explicit dimensions must be preserved.
   const sized=document.createElement('img');sized.id='sized';sized.width=300;sized.height=150;document.body.prepend(sized);window.__cyoaManagerImages.setSource(sized,'src',svg);
  });
  await page.waitForFunction(()=>document.getElementById('img-0').naturalWidth>0);
  // Dimensions of unvisited images are unknown until loaded, so visit once.
  for(let n=0;n<60;n+=3){await page.locator('#img-'+n).scrollIntoViewIfNeeded();await page.waitForTimeout(80);}
  await page.evaluate(()=>scrollTo(0,0));await page.waitForTimeout(200);
  const height=await page.evaluate(()=>document.body.scrollHeight);
  for(let pass=0;pass<3;pass++){
   await page.evaluate(()=>scrollTo(0,document.body.scrollHeight));await page.waitForTimeout(150);
   assert((await page.locator('#img-0').getAttribute('src')).startsWith('data:image/svg+xml,'));
   assert.equal(await page.evaluate(()=>document.body.scrollHeight),height,JSON.stringify(await page.locator('img').evaluateAll(xs=>xs.slice(0,5).map(x=>({src:!!x.getAttribute('src'),w:x.width,h:x.height,rect:x.getBoundingClientRect().height,ratio:getComputedStyle(x).aspectRatio})))));
   assert((await page.locator('img[src^="/__manager_images/"]').count())<12);
   await page.evaluate(()=>scrollTo(0,0));await page.waitForFunction(()=>document.getElementById('img-0').naturalWidth>0);
   assert.equal(await page.evaluate(()=>document.body.scrollHeight),height,JSON.stringify(await page.locator('img').evaluateAll(xs=>xs.slice(0,5).map(x=>({src:!!x.getAttribute('src'),w:x.width,h:x.height,rect:x.getBoundingClientRect().height,ratio:getComputedStyle(x).aspectRatio})))));
  }
  assert.equal(await page.locator('#sized').getAttribute('width'),'300');
  // Match the bundled viewer: percentage width, without height:auto. Adding
  // intrinsic HTML dimensions changes the height calculation in this layout.
  await page.evaluate(()=>{
   const section=document.createElement('section');section.id='responsive';section.style.width='240px';
   section.innerHTML='<img style="display:block;width:98%;max-width:none;margin:0"/><img width="300" height="150" style="display:block;width:98%;max-width:none;margin:0"/>';
   document.body.prepend(section);
   for(const image of section.children)window.__cyoaManagerImages.setSource(image,'src','/__manager_images/fixture.svg?w=800&h=400');
  });
  const responsive=page.locator('#responsive img').first();
  await page.waitForTimeout(150);
  assert.equal(await responsive.getAttribute('width'),null);
  assert.equal(await responsive.getAttribute('height'),null);
  const size=await responsive.evaluate(image=>({width:image.getBoundingClientRect().width,height:image.getBoundingClientRect().height}));
  assert(Math.abs(size.width/size.height-2)<0.01,JSON.stringify(size));
  await page.evaluate(()=>document.getElementById('responsive').style.width='420px');
  const resized=await responsive.evaluate(image=>({width:image.getBoundingClientRect().width,height:image.getBoundingClientRect().height}));
  assert(Math.abs(resized.width/resized.height-2)<0.01,JSON.stringify(resized));
  await page.evaluate(()=>scrollTo(0,document.body.scrollHeight));await page.waitForTimeout(150);
  assert((await responsive.getAttribute('src')).startsWith('data:image/svg+xml,'));
  const unloaded=await responsive.evaluate(image=>image.getBoundingClientRect().height);
  assert(Math.abs(unloaded-resized.height)<1);
  assert.equal(await page.locator('#responsive img').nth(1).getAttribute('width'),'300');
  assert.equal(await page.locator('#responsive img').nth(1).getAttribute('height'),'150');
  await page.evaluate(()=>{document.getElementById('responsive').remove();scrollTo(0,0);});
  // Existing separated artwork/remote URLs must also release after first loading.
  await page.evaluate(()=>{const image=document.createElement('img');image.id='existing-resource';document.body.prepend(image);window.__cyoaManagerImages.setSource(image,'src','/existing.png');});
  await page.waitForFunction(()=>document.getElementById('existing-resource').naturalWidth>0);
  await page.evaluate(()=>scrollTo(0,document.body.scrollHeight));await page.waitForTimeout(150);
  assert((await page.locator('#existing-resource').getAttribute('src')).startsWith('data:image/svg+xml,'));
  await page.evaluate(()=>scrollTo(0,0));await page.waitForTimeout(150);
  await page.evaluate(()=>{const image=document.getElementById('img-0');image.remove();document.body.prepend(image);});
  await page.waitForFunction(()=>document.getElementById('img-0').naturalWidth>0);
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  await page.evaluate(()=>document.body.replaceChildren());await page.waitForTimeout(100);assert.deepEqual(errors,[]);
  console.log('Reader images: viewport release, repeat scrolling, layout, authored sizes and detached nodes passed.');
 } finally {await browser.close();}
})().catch(error=>{console.error(error);process.exitCode=1});
