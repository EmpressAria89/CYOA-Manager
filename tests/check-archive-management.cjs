const {chromium}=require('playwright'),assert=require('node:assert/strict');
(async()=>{const browser=await chromium.launch({headless:true});const page=await browser.newPage({viewport:{width:1280,height:950}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
await page.route('https://api.github.com/**',r=>r.fulfill({status:404,body:'{}'}));
await page.addInitScript(()=>{
 const main={id:'11111111-1111-4111-8111-111111111111',name:'Current Fire Emblem',title:'Current Fire Emblem',author:'Gaston',fandom:'Fire Emblem',tags:[],file_path:'/fixture/current.json',source_url:'https://example.com/',project_json_url:'https://example.com/project.json',viewer_preference:'icc2-plus',date_added:'2026-10-08',file_missing:false,favorite:false,cover_image:null};
 const other={...main,id:'22222222-2222-4222-8222-222222222222',title:'Fire Emblem 2',name:'Fire Emblem 2'};
 const old={id:'33333333-3333-4333-8333-333333333333',created_at:'2026-10-08',reason:'Imported library copy',starred:true,project:{...other,name:'Old Fire Emblem',title:'Old Fire Emblem'}};
 let projects=[main],versions=[old,{...old,id:'44444444-4444-4444-8444-444444444444',project:main,starred:false}];const labels={[main.id]:'Fire Emblem 2',[other.id]:'Fire Emblem 2'};window.__calls=[];
 window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args)=>{
 window.__calls.push({command,args});
 if(command==='get_library')return structuredClone(projects);
 if(command==='update_project'){const p=projects.find(p=>p.id===args.id);Object.assign(p,args.patch);return structuredClone(p);}
 if(command==='get_viewers')return [{id:'icc2-plus',name:'ICC2 Plus'}];
 if(command==='list_archive_groups')return [...new Map(versions.map(v=>[v.project.id,{...v.project,name:labels[v.project.id],title:labels[v.project.id]}])).values()];
 if(command==='list_archives')return structuredClone(versions);
 if(command==='list_versions')return structuredClone(versions.filter(v=>v.project.id===args.id));
 if(command==='rename_archive_group'){labels[args.id]=args.name;return;}
 if(command==='rename_archive_version'){const v=versions.find(v=>v.id===args.versionId);v.project.name=v.project.title=args.name;return;}
 if(command==='restore_version'){let restored=projects.find(p=>p.restored_from_archive);if(!restored){restored={...versions.find(v=>v.id===args.versionId).project,id:'55555555-5555-4555-8555-555555555555',restored_from_archive:true};projects.push(restored);}return structuredClone(restored);}
 if(command==='move_archive_version'){const v=versions.find(v=>v.id===args.versionId);v.project.id=args.targetId;return structuredClone(v);}
 return null;
 }};
});
const url=process.env.CYOA_TEST_URL||'http://127.0.0.1:5797';
await page.goto(url+'/#/archives');await page.getByRole('heading',{name:'Old Fire Emblem',exact:true}).waitFor();assert.equal(await page.locator('.archive-view > section').count(),1);assert.equal(await page.getByRole('heading',{name:'Current Fire Emblem',exact:true}).count(),0);await page.locator('.archive-view > section').filter({hasText:'Edition 33333333'}).getByRole('button',{name:'Browse / open editions'}).click();
await page.getByRole('button',{name:'Rename archive group',exact:true}).click();await page.getByLabel('Archive group name',{exact:true}).fill('Fire Emblem — vanilla');await page.getByRole('button',{name:'Save group name',exact:true}).click();await page.getByRole('heading',{name:'Fire Emblem — vanilla — Archives'}).waitFor();
await page.getByRole('button',{name:'Rename edition',exact:true}).click();await page.getByLabel('Edition name',{exact:true}).fill('Fire Emblem vanilla 2025');await page.getByRole('button',{name:'Save edition name',exact:true}).click();await page.getByText('Edition renamed.',{exact:true}).waitFor();
await page.getByRole('button',{name:'Add copy to library',exact:true}).click();await page.getByRole('status').filter({hasText:'Added “Fire Emblem vanilla 2025”'}).waitFor();
await page.getByRole('button',{name:'Add copy to library',exact:true}).click();await page.getByText('“Fire Emblem vanilla 2025” is already in your library.',{exact:true}).waitFor();
await page.getByRole('button',{name:'Move edition',exact:true}).click();await page.getByLabel('Destination archive group',{exact:true}).selectOption('11111111-1111-4111-8111-111111111111');await page.getByRole('button',{name:'Move to group',exact:true}).click();await page.getByText('No starred editions remain in this group.',{exact:true}).waitFor();
await page.getByRole('button',{name:'Close',exact:true}).click();await page.goto(url+'/#/');await page.getByRole('heading',{name:'Fire Emblem vanilla 2025',exact:true}).waitFor();
const restored=page.locator('.card').filter({has:page.getByRole('heading',{name:'Fire Emblem vanilla 2025',exact:true})});assert.equal(await restored.getByRole('button',{name:'Re-download',exact:true}).count(),0);assert.equal(await restored.getByText('Archived edition',{exact:true}).count(),1);
const main=page.locator('.card').filter({has:page.getByRole('heading',{name:'Current Fire Emblem',exact:true})});assert.equal(await main.getByRole('button',{name:'Re-download',exact:true}).count(),1);assert.equal(await page.locator('.library-view .card').count(),2);
await restored.getByTitle('Options').click();await page.getByRole('menuitem',{name:/Edit/}).click();const dialog=page.getByRole('dialog');const force=dialog.getByRole('button',{name:'Force update',exact:true});await force.waitFor();assert((await force.boundingBox()).x<(await dialog.getByRole('button',{name:'Cancel',exact:true}).boundingBox()).x);
page.once('dialog',d=>d.dismiss());await force.click();assert.equal((await page.evaluate(()=>window.__calls)).filter(c=>c.command==='start_overwrite_catalog_entry').length,0);
page.once('dialog',d=>{assert.match(d.message(),/archived first/);d.accept();});await force.click();await page.waitForFunction(()=>window.__calls.some(c=>c.command==='start_overwrite_catalog_entry'&&c.args.forceUpdate===true));
assert.deepEqual(errors,[]);await browser.close();console.log('Archives: rename groups/editions, move edition, separate named restore, retry feedback, fresh library list and starred-only listing, restored-copy update protection and confirmed force-update action passed (backend mocked).');
})().catch(e=>{console.error(e);process.exit(1)});
