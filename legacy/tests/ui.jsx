(function(){
    var root = new File($.fileName).parent.parent.fsName;
    var report=[];
    try {
        $.global.CHROMATIC_AE_TEST_MODE=true;
        $.evalFile(new File(root + '/Chromatic AE.jsx'));
        $.global.CHROMATIC_AE_TEST_MODE=false;
        var w=$.global.ChromaticAE.ui(this);
        report.push('PASS UI constructed in AE '+app.version+' size '+w.size.toString());
    }catch(e){report.push('FAIL '+e.toString()+' line '+e.line);}
    var f=new File(root + '/test-output/ui-results.txt');f.open('w');f.write(report.join('\n'));f.close();
}());
