(function(){
    app.beginSuppressDialogs();
    var root=new File($.fileName).parent.parent.fsName;
    try{
        app.open(new File(root+'/test-output/native-centre-verification.aep'));
        var comp=null;for(var i=1;i<=app.project.numItems;i++){var p=app.project.item(i);if(p instanceof CompItem && p.name==='Centre restoration test')comp=p;}
        while(app.project.renderQueue.numItems)app.project.renderQueue.item(1).remove();
        for(i=0;i<2;i++){
            var item=app.project.renderQueue.items.add(comp);item.applyTemplate('Best Settings');item.timeSpanStart=0;item.timeSpanDuration=1/24;
            item.setSetting('Resolution',i===0?'Full':'Half');
            item.outputModule(1).applyTemplate('jpg');item.outputModule(1).file=new File(root+'/test-output/centre-'+(i===0?'full':'half')+'.jpg');
        }
        app.project.save(new File(root+'/test-output/centre-resolution.aep'));
        var f=new File(root+'/test-output/centre-render-setup.txt');f.open('w');f.write('READY');f.close();
    }catch(e){var f=new File(root+'/test-output/centre-render-setup.txt');f.open('w');f.write('ERROR '+e.toString()+' line '+e.line);f.close();}
    app.endSuppressDialogs(false);
}());
