(function(){
    app.beginSuppressDialogs();
    $.global.CHROMATIC_NATIVE_HEADLESS=true;
    var root=new File($.fileName).parent.parent.fsName;
    try{
        $.evalFile(new File(root+'/Create Demo.jsx'));
        app.project.bitsPerChannel=32;
        var comp=$.global.CHROMATIC_NATIVE_DEMO;
        var item=app.project.renderQueue.items.add(comp);item.timeSpanStart=1.5;item.timeSpanDuration=1/30;
        item.outputModule(1).applyTemplate('jpg');item.outputModule(1).file=new File(root+'/test-output/native-preview.jpg');
        app.project.save(new File(root+'/examples/Chromatic AE Native Demo.aep'));
        var f=new File(root+'/test-output/native-demo-result.txt');f.open('w');f.write('Saved native demo with '+comp.numLayers+' layers and one adjustment effect.');f.close();
    }catch(e){var f=new File(root+'/test-output/native-demo-result.txt');f.open('w');f.write('ERROR '+e.toString()+' line '+e.line);f.close();}
    app.endSuppressDialogs(false);
}());
