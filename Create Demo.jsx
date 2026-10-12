/* A self-contained animated demo. Run beside Chromatic AE.jsx. */
(function () {
    var oldMode = $.global.CHROMATIC_AE_TEST_MODE;
    try {
        $.global.CHROMATIC_AE_TEST_MODE = true;
        $.evalFile(new File(new File($.fileName).parent.fsName + '/Chromatic AE.jsx'));
        var api = $.global.ChromaticAE;
        app.beginUndoGroup('Create Chromatic AE demo');
        var source = app.project.items.addComp('Chromatic demo - editable artwork',960,540,1,5,30);
        function textLine(text,size,position,color) {
            var l=source.layers.addText(text), p=l.property('ADBE Text Properties').property('ADBE Text Document'), d=p.value;
            d.font='Arial-BoldMT';d.fontSize=size;d.fillColor=color;d.applyFill=true;d.applyStroke=false;
            p.setValue(d);l.property('ADBE Transform Group').property('ADBE Position').setValue(position);return l;
        }
        textLine('CHROMATIC / 01',16,[64,76],[0.55,0.6,0.67]);
        textLine('COLOUR',108,[57,228],[1,1,1]);
        textLine('IN MOTION.',108,[57,336],[1,1,1]);
        textLine('RGB SPLIT     /     CUSTOM COLOURS     /     SOFT EDGES',14,[64,455],[0.75,0.78,0.83]);
        for(var i=0;i<3;i++) {
            var l=source.layers.addShape();l.name='Ring '+(i+1);
            l.property('ADBE Transform Group').property('ADBE Position').setValue([788,228]);
            var contents=l.property('ADBE Root Vectors Group');
            var ellipse=contents.addProperty('ADBE Vector Shape - Ellipse');
            ellipse.property('ADBE Vector Ellipse Size').setValue([74+i*48,74+i*48]);
            var stroke=contents.addProperty('ADBE Vector Graphic - Stroke');
            stroke.property('ADBE Vector Stroke Color').setValue([1,1,1,1]);stroke.property('ADBE Vector Stroke Width').setValue(3);
        }
        var rig=api.build(source,{distance:6,teal:true,blur:0.4});source.parentFolder=rig.folder;
        api.ctrl(rig.controller,'Red Distance (px)').setValueAtTime(0,-2);
        api.ctrl(rig.controller,'Red Distance (px)').setValueAtTime(2,-14);
        api.ctrl(rig.controller,'Red Distance (px)').setValueAtTime(4,-2);
        api.ctrl(rig.controller,'Blue Distance (px)').setValueAtTime(0,2);
        api.ctrl(rig.controller,'Blue Distance (px)').setValueAtTime(2,10);
        api.ctrl(rig.controller,'Blue Distance (px)').setValueAtTime(4,2);
        var demo=app.project.items.addComp('Chromatic AE - DEMO',960,540,1,5,30);
        demo.layers.addSolid([0.025,0.03,0.045],'Background',960,540,1,5);
        var output=demo.layers.add(rig.output);output.name='Chromatic - select and Open controls';output.selected=true;
        demo.time=1.5;demo.openInViewer();
        $.global.CHROMATIC_AE_DEMO={comp:demo,rig:rig};
        if (!$.global.CHROMATIC_AE_HEADLESS_DEMO) api.ui(this);
    } catch(e) { if ($.global.CHROMATIC_AE_HEADLESS_DEMO) throw e; else alert('Demo: '+e.toString()); }
    finally { $.global.CHROMATIC_AE_TEST_MODE=oldMode;app.endUndoGroup(); }
}());
