(function(){
    app.beginSuppressDialogs();
    var root=new File($.fileName).parent.parent.fsName,lines=[],passed=0,failed=0;
    function check(ok,label,v){lines.push((ok?'PASS ':'FAIL ')+label+(v!==undefined?' | '+(v.join?v.join(','):v):''));if(ok)passed++;else failed++;}
    function near(a,b,t){for(var i=0;i<a.length;i++)if(Math.abs(a[i]-b[i])>t)return false;return true;}
    function set(fx,name,v){fx.property(name).setValue(v);}
    function sample(comp,layer,p){var n=comp.layers.addNull(),c=n.property('ADBE Effect Parade').addProperty('ADBE Color Control').property(1);c.expression='thisComp.layer('+layer.index+').sampleImage(['+p.join(',')+'],[0.01,0.01],true,0);';var v=c.valueAtTime(0,false),e=c.expressionError;n.remove();if(e)throw Error(e);return v;}
    try{
        lines.push('Chromatic AE 2.2 bounds/dimensions | After Effects '+app.version);
        var comp=app.project.items.addComp('Expanded source test',256,256,1,2,24);
        var layer=comp.layers.addSolid([1,1,1],'64px source',64,64,1,2);
        var fx=layer.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');
        var wrap=app.project.items.addComp('Expanded source wrapper',256,256,1,2,24);var output=wrap.layers.add(comp);
        set(fx,'Repeat Edge Pixels',0);set(fx,'Red Distance',-20);set(fx,'Blue Distance',20);
        for(var d=0;d<3;d++){
            var depth=[8,16,32][d];app.project.bitsPerChannel=depth;
            var left=sample(wrap,output,[84,128]),right=sample(wrap,output,[172,128]);
            check(near(left,[1,0,0,1],0.01)&&near(right,[0,0,1,1],0.01),'Channels extend beyond physical layer at '+depth+' bpc','left='+left.join(',')+' right='+right.join(','));
            var center=sample(wrap,output,[128,128]);check(near(center,[1,1,1,1],0.01),'Expansion preserves original position at '+depth+' bpc',center);
        }
        set(fx,'Expand Layer Bounds',0);check(sample(wrap,output,[84,128])[3]<0.001,'Bounds switch restores original cropping');set(fx,'Expand Layer Bounds',1);
        set(fx,'Red Distance',0);set(fx,'Blue Distance',0);set(fx,'Edge Softness',8);
        set(fx,'Blur Dimensions',2);var hleft=sample(wrap,output,[90,128]),htop=sample(wrap,output,[128,90]);check(hleft[3]>0.05&&htop[3]<0.001,'Horizontal blur extends horizontally only','left='+hleft.join(',')+' top='+htop.join(','));
        set(fx,'Blur Dimensions',3);var vleft=sample(wrap,output,[90,128]),vtop=sample(wrap,output,[128,90]);check(vleft[3]<0.001&&vtop[3]>0.05,'Vertical blur extends vertically only','left='+vleft.join(',')+' top='+vtop.join(','));
        set(fx,'Blur Dimensions',1);var bleft=sample(wrap,output,[90,128]),btop=sample(wrap,output,[128,90]);check(bleft[3]>0.05&&btop[3]>0.05,'Both dimensions blur beyond every edge','left='+bleft.join(',')+' top='+btop.join(','));
        check(near(bleft,[1,1,1,bleft[3]],0.002),'Extended blur retains straight colour',bleft);
        check(sample(wrap,output,[60,128])[3]<0.001,'Blur tail ends in transparency');
        set(fx,'Blur Dimensions',2);set(fx,'Red Blur Dimensions',4);var rt=sample(wrap,output,[128,90]),rl=sample(wrap,output,[90,128]);check(rt[0]>0.99&&rt[1]<0.001&&rt[2]<0.001&&rt[3]>0.05,'Red override blurs vertically',rt);check(rl[0]<0.001&&rl[1]>0.99&&rl[2]>0.99,'Green and blue inherit horizontal master',rl);
        set(fx,'Green Blur Dimensions',4);set(fx,'Blue Blur Dimensions',4);var allv=sample(wrap,output,[128,90]);check(near(allv,[1,1,1,allv[3]],0.002)&&allv[3]>0.05,'All channel dimension overrides work',allv);
        set(fx,'Mix',0);check(sample(wrap,output,[90,128])[3]<0.001&&near(sample(wrap,output,[128,128]),[1,1,1,1],0.002),'Mix zero bypasses expanded blur');
        set(fx,'Mix',100);set(fx,'Edge Softness',0);set(fx,'Red Distance',-20);set(fx,'Blue Distance',20);set(fx,'Direction',90);var up=sample(wrap,output,[128,84]),down=sample(wrap,output,[128,172]);check(near(up,[1,0,0,1],0.001)&&near(down,[0,0,1,1],0.001),'Separation extends beyond top and bottom','up='+up.join(',')+' down='+down.join(','));
        // A second expanding effect must honour the first effect's negative origin.
        set(fx,'Direction',0);var fx2=layer.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');set(fx2,'Repeat Edge Pixels',0);set(fx2,'Red Distance',-10);set(fx2,'Blue Distance',10);var stacked=sample(wrap,output,[72,128]);check(near(stacked,[1,0,0,1],0.002),'Stacked expanding effects retain origins',stacked);
        // Saved project for render-level full/half-resolution verification.
        app.project.save(new File(root+'/test-output/native-bounds-verification.aep'));
        // In headless AE, reopening a project can leave sampleImage probes stale.
        // Run saved-project metadata checks after all rendered pixel checks.
        if(parseFloat(app.version)>=26){
        app.open(new File(root+'/tests/fixtures/v2.0-demo.aep'));
        var old=null;for(var i=1;i<=app.project.numItems;i++){var it=app.project.item(i);if(it instanceof CompItem && it.name==='Chromatic AE - Native DEMO')old=it;}
        var ofx=old.layer('CHROMATIC AE - select for controls').property('ADBE Effect Parade').property('ChromaticAE RGB Split');
        check(ofx.property('Red Distance').numKeys===3 && ofx.property('Red Distance').keyValue(2)===-12,'2.0 project keeps animated distance');
        check(near(ofx.property('Green Colour').value,[0,0.85,0.65,1],0.001),'2.0 project keeps custom colour',ofx.property('Green Colour').value);
        check(ofx.property('Blur Dimensions').value===1 && ofx.property('Red Blur Dimensions').value===1 && ofx.property('Expand Layer Bounds').value===1 && ofx.property('Centre Colour Restore').value===0,'2.0 project receives compatible new defaults');
        } else { lines.push('SKIP 2.0 project upgrade: fixture requires AE 2026'); }

    }catch(e){failed++;lines.push('FATAL '+e.toString()+' line '+e.line);}
    lines.push('RESULT '+passed+' passed, '+failed+' failed');
    var file=new File(root+'/test-output/native-bounds-results.txt');file.open('w');file.write(lines.join('\n'));file.close();app.endSuppressDialogs(false);
}());
