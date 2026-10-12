/* Native plugin integration: runs in a fresh headless AE instance. */
(function(){
    var root=new File($.fileName).parent.parent.fsName;
    app.beginSuppressDialogs();
    var lines=[],passed=0,failed=0;
    function check(ok,name,value){lines.push((ok?'PASS ':'FAIL ')+name+(value!==undefined?' | '+(value.join?value.join(','):value):''));if(ok)passed++;else failed++;}
    function near(a,b,t){for(var i=0;i<a.length;i++){if(Math.abs(a[i]-b[i])>t)return false;}return true;}
    function find(group,name){for(var i=1;i<=group.numProperties;i++){var p=group.property(i);if(p.name===name)return p;if(p.propertyType!==PropertyType.PROPERTY){var r=find(p,name);if(r)return r;}}return null;}
    function set(effect,name,value){var p=find(effect,name);if(!p)throw Error('Missing control '+name);p.setValue(value);return p;}
    function sample(comp,layer,point,time){time=time||0;var n=comp.layers.addNull();var p=n.property('ADBE Effect Parade').addProperty('ADBE Color Control').property(1);p.expression='thisComp.layer('+layer.index+').sampleImage(['+point.join(',')+'],[0.01,0.01],true,'+time+');';var v=p.valueAtTime(time,false);var error=p.expressionError;n.remove();if(error)throw Error(error);return v;}
    function dump(group,prefix){for(var i=1;i<=group.numProperties;i++){var p=group.property(i);lines.push(prefix+p.name+' ('+p.matchName+')');if(p.propertyType!==PropertyType.PROPERTY)dump(p,prefix+'  ');}}
    try{
        lines.push('After Effects '+app.version+' | Chromatic AE native 2.0');
        var comp=app.project.items.addComp('Native adjustment test',128,128,1,2,24);
        comp.layers.addSolid([0,0,0],'Background',128,128,1,2);
        comp.layers.addSolid([1,1,1],'White square',64,64,1,2);
        var green=comp.layers.addSolid([0,1,0],'Green square',16,16,1,2);green.position.setValue([96,24]);
        var original=comp.duplicate();original.name='Unprocessed reference';
        var adj=comp.layers.addSolid([1,1,1],'Chromatic AE adjustment',128,128,1,2);adj.adjustmentLayer=true;
        var fx=adj.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');
        dump(fx,'');
        check(adj.adjustmentLayer && adj.property('ADBE Effect Parade').numProperties===1,'Exactly one native effect on adjustment layer');
        var wrapper=app.project.items.addComp('Native sample wrapper',128,128,1,2,24);
        var output=wrapper.layers.add(comp);var reference=wrapper.layers.add(original);reference.enabled=false;
        set(fx,'Red Distance',0);set(fx,'Blue Distance',0);
        for(var d=0;d<3;d++){
            var depth=[8,16,32][d];app.project.bitsPerChannel=depth;
            var a=sample(wrapper,reference,[64,64]);var b=sample(wrapper,output,[64,64]);
            check(near(a,b,0.005),'Adjustment identity '+depth+' bpc',b);
            set(fx,'Red Distance',-12);set(fx,'Blue Distance',12);
            var left=sample(wrapper,output,[25,64]);var right=sample(wrapper,output,[102,64]);
            check(near(left,[1,0,0,1],0.01) && near(right,[0,0,1,1],0.01),'Adjustment split '+depth+' bpc','left='+left.join(',')+' right='+right.join(','));
            set(fx,'Red Distance',0);set(fx,'Blue Distance',0);
        }
        app.project.bitsPerChannel=32;
        set(fx,'Green Colour',[0,0.8,0.7,1]);var teal=sample(wrapper,output,[96,24]);
        check(near(teal,[0,0.8,0.7,1],0.01),'Green contribution recolours to teal',teal);
        set(fx,'Green Colour',[0,1,0,1]);
        var red=find(fx,'Red Distance');red.setValueAtTime(0,0);red.setValueAtTime(1,20);
        var at0=sample(wrapper,output,[102,64],0);var at1=sample(wrapper,output,[102,64],1);
        check(at0[0]<0.01 && at1[0]>0.99,'Independent red distance keyframes render','t0='+at0.join(',')+' t1='+at1.join(','));
        var midpoint=sample(wrapper,output,[100,64],0.5);check(midpoint[0]>0.99,'Distance keyframes interpolate',midpoint);
        red.removeKey(2);red.removeKey(1);red.setValue(12);
        set(fx,'Direction',90);var rotated=sample(wrapper,output,[64,102]);
        check(near(rotated,[1,0,0,1],0.01),'Direction rotates split',rotated);
        set(fx,'Direction',0);set(fx,'Red Distance',0);
        var hard=sample(wrapper,output,[29,64]);set(fx,'Edge Softness',6);var soft=sample(wrapper,output,[29,64]);
        check(hard[0]<0.001 && soft[0]>0.05 && soft[0]<0.5,'Global edge softness renders',soft);
        set(fx,'Edge Softness',0);set(fx,'Red Softness',6);var redSoft=sample(wrapper,output,[29,64]);
        check(redSoft[0]>0.05 && redSoft[1]<0.001 && redSoft[2]<0.001,'Independent red softness renders',redSoft);
        set(fx,'Mix',0);var bypass=sample(wrapper,output,[29,64]);check(near(bypass,hard,0.001),'Mix zero bypasses blur',bypass);
        set(fx,'Mix',100);set(fx,'Red Softness',0);set(fx,'Red Distance',-12);
        adj.opacity.setValue(50);var half=sample(wrapper,output,[25,64]);check(near(half,[0.5,0,0,1],0.02),'Adjustment opacity blends the native effect',half);adj.opacity.setValue(100);
        set(fx,'Split Amount',0);var collapsed=sample(wrapper,output,[25,64]);check(near(collapsed,[0,0,0,1],0.001),'Master amount collapses separation',collapsed);set(fx,'Split Amount',100);
        // A direct layer with translucent pixels exercises premultiplied alpha.
        var alpha=app.project.items.addComp('Native alpha',128,128,1,2,24);
        var patch=alpha.layers.addSolid([0.2,0.5,0.1],'Translucent patch',64,64,1,2);patch.opacity.setValue(50);
        var alphaWrap=app.project.items.addComp('Native alpha wrapper',128,128,1,2,24);var al=alphaWrap.layers.add(alpha);
        var afx=al.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');set(afx,'Red Distance',0);set(afx,'Blue Distance',0);set(afx,'Repeat Edge Pixels',0);
        for(d=0;d<3;d++){depth=[8,16,32][d];app.project.bitsPerChannel=depth;var v=sample(alphaWrap,al,[64,64]);check(near(v,[0.2,0.5,0.1,0.5],depth===8?0.012:0.002),'Premultiplied alpha identity '+depth+' bpc',v);}
        var clear=sample(alphaWrap,al,[5,5]);check(clear[3]<0.001,'Transparent pixels remain clear',clear);
        set(afx,'Red Distance',-12);var fringe=sample(alphaWrap,al,[25,64]);check(near(fringe,[0.2,0,0,0.5],0.003),'Direct effect expands channel into transparent pixels',fringe);
        set(afx,'Red Distance',0);set(afx,'Edge Softness',6);var feather=sample(alphaWrap,al,[29,64]);
        var finalComp=app.project.items.addComp('Alpha final composite',128,128,1,2,24);var finalLayer=finalComp.layers.add(alphaWrap);var compositeFeather=sample(finalComp,finalLayer,[29,64]);lines.push('DIAGNOSTIC composite feather '+compositeFeather.join(','));
        check(feather[3]>0.02 && feather[3]<0.4 && near(feather,[0.2,0.5,0.1,feather[3]],0.005),'Soft alpha avoids dark edges',feather);
                set(fx,'Red Distance',0);set(fx,'Blue Distance',0);
        var gd=find(fx,'Green Distance');gd.setValueAtTime(0,0);gd.setValueAtTime(1,20);
        var gt0=sample(wrapper,output,[102,64],0),gt1=sample(wrapper,output,[102,64],1);
        check(gt0[1]<0.01 && gt1[1]>0.99,'Independent green distance keyframes render',gt1);gd.removeKey(2);gd.removeKey(1);gd.setValue(0);
        var bd=find(fx,'Blue Distance');bd.setValueAtTime(0,0);bd.setValueAtTime(1,20);
        var bt0=sample(wrapper,output,[102,64],0),bt1=sample(wrapper,output,[102,64],1);
        check(bt0[2]<0.01 && bt1[2]>0.99,'Independent blue distance keyframes render',bt1);bd.removeKey(2);bd.removeKey(1);bd.setValue(0);
        set(fx,'Red X Offset',12);set(fx,'Red Y Offset',12);var xy=sample(wrapper,output,[102,102]);check(near(xy,[1,0,0,1],0.01),'Independent X and Y offsets render',xy);set(fx,'Red X Offset',0);set(fx,'Red Y Offset',0);
        set(fx,'Green Colour',[0,1,2,1]);var hdr=sample(wrapper,output,[64,64]);check(hdr[2]>2.99,'32-bit HDR colour is not clipped',hdr);set(fx,'Green Colour',[0,1,0,1]);
        set(fx,'Red Distance',-12);set(fx,'Blue Distance',12);
        var mask=adj.property('ADBE Mask Parade').addProperty('ADBE Mask Atom');var shape=new Shape();shape.vertices=[[0,0],[64,0],[64,128],[0,128]];shape.closed=true;shape.inTangents=[[0,0],[0,0],[0,0],[0,0]];shape.outTangents=shape.inTangents;mask.property('ADBE Mask Shape').setValue(shape);
        var maskedLeft=sample(wrapper,output,[25,64]);var maskedRight=sample(wrapper,output,[102,64]);check(maskedLeft[0]>0.99 && maskedRight[2]<0.01,'Adjustment mask limits the effect','left='+maskedLeft.join(',')+' right='+maskedRight.join(','));mask.remove();
        app.project.save(new File(root+'/test-output/native-verification.aep'));
    }catch(e){failed++;lines.push('FATAL '+e.toString()+' line '+e.line);}
    lines.push('RESULT '+passed+' passed, '+failed+' failed');
    var f=new File(root+'/test-output/native-results.txt');f.open('w');f.write(lines.join('\n'));f.close();
    app.endSuppressDialogs(false);
}());
