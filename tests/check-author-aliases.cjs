require('node:fs').mkdirSync('tests/results',{recursive:true});
const {chromium}=require('playwright'),assert=require('node:assert/strict');
const URL=process.env.CYOA_TEST_URL||'http://127.0.0.1:5697';
(async()=>{
 const browser=await chromium.launch({headless:true});const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.route('https://api.github.com/**',r=>r.fulfill({status:404,body:'{}'}));
 await page.route('https://sheets.googleapis.com/**',r=>r.fulfill({json:{values:[['Title','Interactive','Author'],['New story','https://example.com/','Gaston1597']]}}));
 await page.addInitScript(()=>{
  const seed=[{id:'ac9679c6-f8a3-4fa9-b077-5a2a92f3d522',name:'Gaston',aliases:['Gaston1231']}];
  const project={id:'test',name:'Story',title:'Story',author:'Gaston1231',modder:'',fandom:'',is_mod:false,build_count:0,description:'',cover_image:null,file_path:'/fixture/project.json',viewer_preference:'icc2-plus',favorite:false,exclude_from_perk_index:false,date_added:'2026-10-07',tags:[],file_missing:false};
  window.__aliasWrites=[];window.__rawAuthor=project.author;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,unregisterCallback:()=>{},invoke:async(command,args)=>{
   if(command==='get_author_aliases')return JSON.parse(localStorage.getItem('test-author-aliases')||JSON.stringify(seed));
   if(command==='save_author_aliases'){window.__aliasWrites.push(args);localStorage.setItem('test-author-aliases',JSON.stringify(args.authors));return args.authors;}
   if(command==='get_library')return [project];if(command==='get_viewers')return [{id:'icc2-plus',name:'ICC2 Plus'}];if(command==='get_fonts')return ['system-ui'];
   if(command==='update_project'){window.__rawAuthor=args.patch.author;return {...project,...args.patch};}return null;
  }};
 });
 await page.goto(URL+'/#/');await page.locator('.card .author').getByText('Gaston',{exact:true}).waitFor();
 await page.getByTitle('Options').first().click();await page.getByRole('menuitem',{name:/Edit/}).click();await page.getByRole('dialog').getByRole('button',{name:'Save',exact:true}).click();assert.equal(await page.evaluate(()=>window.__rawAuthor),'Gaston1231');
 await page.getByRole('link',{name:'Author aliases',exact:true}).click();await page.getByText('Gaston1231',{exact:true}).waitFor();await page.getByLabel('Another source name').fill('Gaston1597');await page.getByRole('button',{name:'Add alias',exact:true}).click();await page.getByRole('button',{name:'Save aliases',exact:true}).click();await page.getByRole('status').waitFor();
 assert.deepEqual(await page.evaluate(()=>window.__aliasWrites[0].authors[0].aliases),['Gaston1231','Gaston1597']);
 await page.reload();await page.getByText('Gaston1597',{exact:true}).waitFor();assert.equal(await page.getByText('Gaston1231',{exact:true}).count(),1);
 await page.screenshot({path:'tests/results/author-aliases.png'});
 await page.getByRole('link',{name:/Infaera Catalog/}).click();await page.locator('.author-name').getByText('Gaston',{exact:true}).waitFor();await page.getByRole('combobox',{name:'Filter by author'}).selectOption('Gaston');assert.equal(await page.locator('.catalog-view .card').count(),1);
 assert.deepEqual(errors,[]);await browser.close();console.log('Author aliases: persistent additive aliases, immediate library/catalog/filter mapping, original credit preserved on edit passed (backend mocked).');
})().catch(e=>{console.error(e);process.exit(1)});
