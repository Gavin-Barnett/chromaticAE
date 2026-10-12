/* Optional editable native-effect demo. Install Chromatic AE.aex first. */
(function(){
    app.beginUndoGroup('Create Chromatic AE native demo');
    try{
        var comp=app.project.items.addComp('Chromatic AE - Native DEMO',960,540,1,5,30);
        comp.layers.addSolid([0.018,0.024,0.035],'Background',960,540,1,5);
        function textLine(text,size,position,colour){
            var l=comp.layers.addText(text),p=l.property('ADBE Text Properties').property('ADBE Text Document'),d=p.value;
            d.font='Arial-BoldMT';d.fontSize=size;d.fillColor=colour;d.applyFill=true;d.applyStroke=false;
            p.setValue(d);l.position.setValue(position);return l;
        }
        textLine('CHROMATIC AE / 02',16,[64,76],[0.55,0.65,0.7]);
        textLine('COLOUR',108,[57,228],[1,1,1]);
        textLine('IN MOTION.',108,[57,336],[1,1,1]);
        textLine('ONE EFFECT   /   RGB CONTROL   /   SOFT EDGES',14,[64,455],[0.72,0.8,0.83]);
        for(var i=0;i<3;i++){
            var l=comp.layers.addShape();l.name='Ring '+(i+1);l.position.setValue([790,195]);
            var contents=l.property('ADBE Root Vectors Group');
            var ellipse=contents.addProperty('ADBE Vector Shape - Ellipse');ellipse.property('ADBE Vector Ellipse Size').setValue([50+i*42,50+i*42]);
            var stroke=contents.addProperty('ADBE Vector Graphic - Stroke');stroke.property('ADBE Vector Stroke Color').setValue([1,1,1,1]);stroke.property('ADBE Vector Stroke Width').setValue(3);
        }
        var adjustment=comp.layers.addSolid([1,1,1],'CHROMATIC AE - select for controls',960,540,1,5);adjustment.adjustmentLayer=true;
        var fx=adjustment.property('ADBE Effect Parade').addProperty('ChromaticAE RGB Split');
        fx.property('Centre Colour Restore').setValue(85);
        fx.property('Green Colour').setValue([0,0.85,0.65,1]);
        fx.property('Red Distance').setValueAtTime(0,-1);fx.property('Red Distance').setValueAtTime(2,-12);fx.property('Red Distance').setValueAtTime(4,-1);
        fx.property('Blue Distance').setValueAtTime(0,1);fx.property('Blue Distance').setValueAtTime(2,9);fx.property('Blue Distance').setValueAtTime(4,1);
        fx.property('Red Softness').setValue(0.6);fx.property('Blue Softness').setValue(0.3);
        for(i=1;i<=comp.numLayers;i++)comp.layer(i).selected=false;
        adjustment.selected=true;comp.time=1.5;
        if(!$.global.CHROMATIC_NATIVE_HEADLESS)comp.openInViewer();
        $.global.CHROMATIC_NATIVE_DEMO=comp;
    }catch(e){if($.global.CHROMATIC_NATIVE_HEADLESS)throw e;else alert('Install the native Chromatic AE plugin first.\n'+e.toString());}
    finally{app.endUndoGroup();}
}());
