/* Run in a fresh AE test session, not in a production project. */
(function () {
    var root = new File($.fileName).parent.parent.fsName;
    app.beginSuppressDialogs();
    $.global.CHROMATIC_AE_TEST_MODE = true;
    $.evalFile(new File(root + '/Chromatic AE.jsx'));
    var api = $.global.ChromaticAE, lines = [], pass = 0, fail = 0;
    function log(s) { lines.push(s); }
    function check(ok, label, detail) { if (detail && detail.join) detail = detail.join(','); log((ok ? 'PASS ' : 'FAIL ') + label + (detail ? ': ' + detail : '')); if (ok) pass++; else fail++; }
    function near(a, b, tolerance) { if (a.length !== b.length) return false; for (var i=0; i<a.length; i++) if (Math.abs(a[i]-b[i])>tolerance) return false; return true; }
    function sample(comp, layer, point, time) {
        var probe = comp.layers.addNull(comp.duration);
        var e = probe.property('ADBE Effect Parade').addProperty('ADBE Color Control');
        var p = e.property(1);
        p.expression = 'thisComp.layer(' + layer.index + ').sampleImage([' + point.join(',') + '],[0.01,0.01],true,' + (time || 0) + ');';
        var value = p.valueAtTime(time || 0, false);
        var err = p.expressionError;
        probe.remove();
        if (err) throw new Error(err);
        return value;
    }
    function expressionErrors(group, where) {
        for (var i=1; i<=group.numProperties; i++) {
            var p=group.property(i);
            if (p.propertyType === PropertyType.PROPERTY) {
                if (p.canSetExpression && p.expression !== '') { try { p.valueAtTime(0.5,false); } catch (_) {} if (p.expressionError) log('EXPRESSION ERROR '+where+'/'+p.name+': '+p.expressionError); }
            } else expressionErrors(p, where+'/'+p.name);
        }
    }
    try {
        log('After Effects '+app.version+' | Chromatic AE '+api.version);
        var source=app.project.items.addComp('TEST source',128,128,1,2,24);
        var patch=source.layers.addSolid([0.2,0.5,0.1],'patch',64,64,1,2);
        patch.property('ADBE Transform Group').property('ADBE Opacity').setValue(50);
        var rig=api.build(source,{distance:0});
        var test=app.project.items.addComp('TEST samples',128,128,1,2,24);
        var original=test.layers.add(source); original.name='original'; original.enabled=false;
        var processed=test.layers.add(rig.output); processed.name='processed';
        for (var k=0;k<3;k++) {
            var depth=[8,16,32][k]; app.project.bitsPerChannel=depth;
            var a=sample(test,original,[64,64]); var b=sample(test,processed,[64,64]);
            check(near(a,b,depth===8?0.018:0.0015),'Identity with half alpha at '+depth+' bpc','source='+a.join(',')+' output='+b.join(','));
        }
        app.project.bitsPerChannel=32;
        var clear=sample(test,processed,[10,10]);
        check(clear[3]<0.001,'Transparent background retained',clear);
        api.ctrl(rig.controller,'Green Colour').setValue([0,0.8,0.7,1]);
        var teal=sample(test,processed,[64,64]);
        check(near(teal,[0.2,0.4,0.45,0.5],0.002),'Green channel remaps to teal',teal);
        api.ctrl(rig.controller,'Green Colour').setValue([0,1,0,1]);
        api.ctrl(rig.controller,'Red Distance (px)').setValue(-12);
        api.ctrl(rig.controller,'Blue Distance (px)').setValue(12);
        var left=sample(test,processed,[25,64]); var right=sample(test,processed,[105,64]);
        check(near(left,[0.2,0,0,0.5],0.002),'Red fringe outside original alpha',left);
        check(near(right,[0,0,0.1,0.5],0.002),'Blue fringe outside original alpha',right);
        api.ctrl(rig.controller,'Red Distance (px)').setValue(0);
        api.ctrl(rig.controller,'Blue Distance (px)').setValue(0);
        var sharp=sample(test,processed,[30,64]);
        api.ctrl(rig.controller,'Edge Softness (px)').setValue(6);
        var soft=sample(test,processed,[30,64]);
        check(near(soft,[0.2,0.5,0.1,soft[3]],0.004),'Soft edges retain straight colour',soft);
        check(sharp[3]<0.001 && soft[3]>0.01 && soft[3]<0.5,'Master softness feathers edges','sharp='+sharp.join(',')+' soft='+soft.join(','));
        api.ctrl(rig.controller,'Edge Softness (px)').setValue(0);
        api.ctrl(rig.controller,'Red Softness (px)').setValue(6);
        var redSoft=sample(test,processed,[30,64]);
        check(redSoft[0]>0.05 && redSoft[1]<0.002 && redSoft[2]<0.002 && redSoft[3]>0.01,'Independent red softness',redSoft);
        api.ctrl(rig.controller,'Red Softness (px)').setValue(0);
        var red=api.ctrl(rig.controller,'Red Distance (px)'); red.setValueAtTime(0,0);red.setValueAtTime(1,20);
        var pos=rig.rgb.layer('Red contribution').property('ADBE Transform Group').property('ADBE Position').valueAtTime(0.5,false);
        check(near(pos,[74,64,0],0.001),'Animated distance interpolates',pos);
        api.ctrl(rig.controller,'Direction').setValue(90);
        pos=rig.rgb.layer('Red contribution').property('ADBE Transform Group').property('ADBE Position').valueAtTime(0.5,false);
        check(near(pos,[64,74,0],0.001),'Direction rotates the split',pos);
        api.ctrl(rig.controller,'Split Amount (%)').setValue(0);
        pos=rig.rgb.layer('Red contribution').property('ADBE Transform Group').property('ADBE Position').valueAtTime(0.5,false);
        check(near(pos,[64,64,0],0.001),'Master amount collapses offsets',pos);
        for (k=0;k<3;k++) { var c=[rig.output,rig.rgb,rig.matte][k];for(var j=1;j<=c.numLayers;j++) expressionErrors(c.layer(j),c.name+'/'+c.layer(j).name); }
        var edit=app.project.items.addComp('TEST apply',128,128,1,4,24);
        var item=edit.layers.add(source); item.startTime=1;item.inPoint=1.25;item.outPoint=2.5;
        var applied=api.apply(edit,[item],{distance:0});
        check(edit.numLayers===1 && applied.wrapper.inPoint===1.25 && applied.wrapper.outPoint===2.5,'Apply preserves trim and one output layer');
        check(applied.source.layer(1).startTime===1,'Source timing retained');
        var unsupported=edit.layers.addNull();
        var rejected=false;try{api.validate(edit,[unsupported]);}catch(_){rejected=true;}
        check(rejected,'Unsupported layers rejected before modification');
        var rq=app.project.renderQueue.items.add(test);rq.timeSpanStart=0;rq.timeSpanDuration=1/24;
        rq.outputModule(1).applyTemplate('TIFF Sequence with Alpha');
        rq.outputModule(1).file=new File(root + '/test-output/identity.tif');
        app.project.save(new File(root + '/test-output/verification.aep'));
        log('Saved verification.aep');
    } catch(error) { fail++;log('FATAL '+error.toString()+' line '+error.line); }
    log('RESULT '+pass+' passed, '+fail+' failed');
    var f=new File(root + '/test-output/results.txt');f.open('w');f.write(lines.join('\n'));f.close();
    app.endSuppressDialogs(false);
}());
