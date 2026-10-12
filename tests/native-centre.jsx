/* Colour restoration checks run before any app.open calls (headless sampleImage limitation). */
(function(){
    app.beginSuppressDialogs();
    var root=new File($.fileName).parent.parent.fsName,lines=[],passed=0,failed=0;
    function check(ok,label,v){lines.push((ok?'PASS ':'FAIL ')+label+(v!==undefined?' | '+(v.join?v.join(','):v):''));if(ok)passed++;else failed++;}
    function near(a,b,t){for(var i=0;i<a.length;i++)if(Math.abs(a[i]-b[i])>t)return false;return true;}
    function set(fx,name,v){fx.property(name).setValue(v);}
    function sample(comp,layer,p,time){time=time||0;var n=comp.layers.addNull(),c=n.property('ADBE Effect Parade').addProperty('ADBE Color Control').property(1);c.expression='thisComp.layer('+layer.index+').sampleImage(['+p.join(',')+'],[0.01,0.01],true,'+time+');';var v=c.valueAtTime(time,false),e=c.expressionError;n.remove();if(e)throw Error(e);return v;}
    try{
        lines.push('Chromatic AE 2.2 centre restoration | After Effects '+app.version);
        var comp=app.project.items.addComp('Centre restoration test',256,192,1,2,24);
        comp.layers.addSolid([0.2,0.5,0.1],'Background',256,192,1,2);
        comp.layers.addSolid([0,1,0],'Fine centre detail',8,32,1,2);
        var original=comp.duplicate();original.name='Centre restoration original';
        var adj=comp.layers.addSolid([1,1,1],'Chromatic adjustment',256,192,1,2);adj.adjustmentLayer=true;
        var fx=adj.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');
        var wrap=app.project.items.addComp('Centre sample wrapper',256,192,1,2,24);
        var output=wrap.layers.add(comp),reference=wrap.layers.add(original);reference.enabled=false;
        check(fx.property('Centre Colour Restore').value===0,'Default zero preserves existing looks');
        set(fx,'Red Distance',-12);set(fx,'Blue Distance',12);set(fx,'Green Colour',[0,0.8,0.7,1]);
        for(var d=0;d<3;d++){
            var depth=[8,16,32][d];app.project.bitsPerChannel=depth;
            set(fx,'Centre Colour Restore',0);var edge=sample(wrap,output,[8,96]),altered=sample(wrap,output,[128,96]);
            set(fx,'Centre Colour Restore',100);var centre=sample(wrap,output,[128,96]),ref=sample(wrap,reference,[128,96]);
            check(near(centre,ref,0.006)&&!near(altered,ref,0.1),'Original centre colour and detail on adjustment layer at '+depth+' bpc',centre);
            check(near(sample(wrap,output,[8,96]),edge,0.006),'Outer edge keeps aberration at '+depth+' bpc',edge);
        }
        app.project.bitsPerChannel=32;
        set(fx,'Centre Colour Restore',0);var zero=sample(wrap,output,[128,96]);
        set(fx,'Centre Colour Restore',50);var half=sample(wrap,output,[128,96]),expected=[];
        for(var k=0;k<4;k++)expected.push((zero[k]+ref[k])/2);
        check(near(half,expected,0.002),'50 percent restores half the centre treatment',half);
        set(fx,'Centre Colour Restore',100);var shoulder=sample(wrap,output,[192,96]),shoulderRef=sample(wrap,reference,[192,96]);
        set(fx,'Centre Colour Restore',0);var shoulderZero=sample(wrap,output,[192,96]);
        check(shoulder[2]>shoulderRef[2]+0.01&&shoulder[2]<shoulderZero[2]-0.01,'Smooth transition between original centre and treated edge',shoulder);
        set(fx,'Centre Colour Restore',100);set(fx,'Mix',0);
        check(near(sample(wrap,output,[192,96]),shoulderRef,0.002),'Mix zero still gives original image');set(fx,'Mix',100);
        var restore=fx.property('Centre Colour Restore');restore.setValueAtTime(0,0);restore.setValueAtTime(1,100);
        check(near(sample(wrap,output,[128,96],0),zero,0.002)&&near(sample(wrap,output,[128,96],1),ref,0.002),'Centre restoration keyframes render');restore.removeKey(2);restore.removeKey(1);restore.setValue(100);
        set(fx,'Edge Softness',12);check(near(sample(wrap,output,[128,96]),ref,0.002),'Centre restores detail lost to blur');
        var direct=app.project.items.addComp('Restoration with expanded bounds',256,256,1,2,24);
        var patch=direct.layers.addSolid([1,1,1],'64px patch',64,64,1,2);
        var dfx=patch.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');set(dfx,'Repeat Edge Pixels',0);set(dfx,'Red Distance',-20);set(dfx,'Blue Distance',20);set(dfx,'Green Colour',[0,0.8,0.7,1]);
        var dw=app.project.items.addComp('Restoration bounds wrapper',256,256,1,2,24),dl=dw.layers.add(direct);
        var fringe=sample(dw,dl,[84,128]);set(dfx,'Centre Colour Restore',100);
        check(near(sample(dw,dl,[128,128]),[1,1,1,1],0.002),'Direct layer centre restores within expanded output');
        check(near(sample(dw,dl,[84,128]),fringe,0.002)&&fringe[0]>0.99,'Restoration retains fringe beyond source bounds');
        set(dfx,'Centre Colour Restore',0);set(dfx,'Green Colour',[0,1,0,1]);set(dfx,'Red Distance',-80);
        var upstream=sample(dw,dl,[128,128]);
        var second=patch.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');set(second,'Red Distance',0);set(second,'Blue Distance',0);set(second,'Green Colour',[0,0.8,0.7,1]);set(second,'Repeat Edge Pixels',0);set(second,'Centre Colour Restore',100);
        check(near(sample(dw,dl,[128,128]),upstream,0.002),'Centre stays aligned after upstream asymmetric expansion',sample(dw,dl,[128,128]));
        app.project.save(new File(root+'/test-output/native-centre-verification.aep'));
    }catch(e){failed++;lines.push('FATAL '+e.toString()+' line '+e.line);}
    lines.push('RESULT '+passed+' passed, '+failed+' failed');
    var file=new File(root+'/test-output/native-centre-results.txt');file.open('w');file.write(lines.join('\n'));file.close();app.endSuppressDialogs(false);
}());
