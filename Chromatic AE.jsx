/* Chromatic AE 1.0.0 | Dockable ScriptUI panel | No external render dependencies. */
(function (host) {
    var VERSION = '1.0.0';
    var TAG = 'chromaticAE:v1';
    var CONTROL = 'CHROMATIC - animate these controls';
    var CHANNELS = ['Red', 'Green', 'Blue'];
    var COLORS = [[1, 0, 0, 1], [0, 1, 0, 1], [0, 0, 1, 1]];
    function quote(s) { return '"' + s.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\r/g, '\\r').replace(/\n/g, '\\n') + '"'; }
    function effect(layer, match, name) {
        var e = layer.property('ADBE Effect Parade').addProperty(match);
        if (name) e.name = name;
        return e;
    }
    function control(layer, name, type, value) {
        var e = effect(layer, 'ADBE ' + type + ' Control', name);
        e.property(1).setValue(value);
    }
    function ctrl(layer, name) { return layer.property('ADBE Effect Parade').property(name).property(1); }
    function addLayer(comp, item, name) {
        var l = comp.layers.add(item); l.startTime = 0; l.inPoint = 0; l.outPoint = comp.duration;
        if (name) l.name = name;
        l.audioEnabled = false;
        return l;
    }
    function newComp(source, name, folder) {
        var c = app.project.items.addComp(name, source.width, source.height, source.pixelAspect, source.duration, source.frameRate);
        c.parentFolder = folder;
        c.displayStartTime = source.displayStartTime;
        c.bgColor = [0, 0, 0];
        c.motionBlur = false;
        return c;
    }
    function blackBackground(comp, folder) {
        var b = comp.layers.addSolid([0, 0, 0], 'Black backing', comp.width, comp.height, comp.pixelAspect, comp.duration);
        b.startTime = 0; b.inPoint = 0; b.outPoint = comp.duration;
        b.source.parentFolder = folder;
        b.locked = true;
        return b;
    }
    function controllerPrefix(output) {
        return 'var c = comp(' + quote(output.name) + ').layer(' + quote(CONTROL) + ');\n';
    }
    function positionExpression(prefix, channel) {
        return prefix + 'var amount = c.effect("Split Amount (%)")(1) / 100;\n' +
            'var d = c.effect("' + channel + ' Distance (px)")(1) * amount;\n' +
            'var a = c.effect("Direction")(1) * Math.PI / 180;\n' +
            'var o = c.effect("' + channel + ' XY Offset")(1);\n' +
            '[thisComp.width/2 + Math.cos(a)*d + o[0]*amount, thisComp.height/2 + Math.sin(a)*d + o[1]*amount];';
    }
    function addBlur(layer, prefix, channel) {
        var e = effect(layer, 'ADBE Gaussian Blur 2', 'Channel edge softness');
        e.property(1).expression = prefix + 'Math.max(0, c.effect("Edge Softness (px)")(1) + c.effect("' + channel + ' Softness (px)")(1));';
        e.property(2).setValue(1);
        e.property(3).setValue(1);
    }
    function build(source, options) {
        options = options || {};
        var distance = options.distance === undefined ? 8 : options.distance;
        var folder = app.project.items.addFolder('Chromatic AE - ' + source.name);
        var suffix = ' [' + folder.id + ']';
        var output = newComp(source, 'Chromatic' + suffix, folder);
        output.comment = TAG + ':output';
        var rgb = newComp(source, 'Chromatic RGB' + suffix, folder);
        rgb.comment = TAG + ':rgb';
        var matte = newComp(source, 'Chromatic Alpha' + suffix, folder);
        matte.comment = TAG + ':alpha';
        var ctl = output.layers.addNull(source.duration);
        ctl.name = CONTROL; ctl.comment = TAG + ':controls'; ctl.label = 14;
        ctl.startTime = 0; ctl.inPoint = 0; ctl.outPoint = source.duration;
        ctl.source.parentFolder = folder;
        control(ctl, 'Split Amount (%)', 'Slider', 100);
        control(ctl, 'Direction', 'Angle', options.angle || 0);
        control(ctl, 'Edge Softness (px)', 'Slider', options.blur || 0);
        for (var i = 0; i < 3; i++) {
            var channel = CHANNELS[i];
            control(ctl, channel + ' Distance (px)', 'Slider', (i - 1) * distance);
            control(ctl, channel + ' XY Offset', 'Point', [0, 0]);
            control(ctl, channel + ' Colour', 'Color', i === 1 && options.teal ? [0, 0.8, 0.7, 1] : COLORS[i]);
            control(ctl, channel + ' Softness (px)', 'Slider', 0);
        }
        var prefix = controllerPrefix(output);
        blackBackground(rgb, folder);
        blackBackground(matte, folder);
        for (i = 0; i < 3; i++) {
            channel = CHANNELS[i];
            var l = addLayer(rgb, source, channel + ' contribution');
            l.blendingMode = BlendingMode.ADD;
            // Replicate exactly one input channel to grayscale, then colour it.
            // Tinting an isolated coloured channel directly would use luminance weights.
            var shift = effect(l, 'ADBE Shift Channels', 'Extract ' + channel);
            shift.property(1).setValue(1);
            shift.property(2).setValue(i + 2);
            shift.property(3).setValue(i + 2);
            shift.property(4).setValue(i + 2);
            var tint = effect(l, 'ADBE Tint', 'Remap ' + channel);
            tint.property(1).setValue([0, 0, 0, 1]);
            tint.property(2).expression = prefix + 'c.effect("' + channel + ' Colour")(1);';
            tint.property(3).setValue(100);
            // Composite before blur so translucent edges are filtered only once.
            var backing = effect(l, 'ADBE Solid Composite', 'Premultiply onto black');
            backing.property(2).setValue([0, 0, 0, 1]);
            backing.property(3).setValue(100);
            addBlur(l, prefix, channel);
            l.property('ADBE Transform Group').property('ADBE Position').expression = positionExpression(prefix, channel);
            // A separate coverage pass takes the maximum shifted/blurred alpha.
            var a = addLayer(matte, source, channel + ' coverage');
            a.blendingMode = BlendingMode.LIGHTEN;
            shift = effect(a, 'ADBE Shift Channels', 'Extract coverage');
            shift.property(1).setValue(9); // Full On
            shift.property(2).setValue(1); // Alpha
            shift.property(3).setValue(1);
            shift.property(4).setValue(1);
            addBlur(a, prefix, channel);
            a.property('ADBE Transform Group').property('ADBE Position').expression = positionExpression(prefix, channel);
        }
        var alphaLayer = addLayer(output, matte, 'Coverage - internal');
        alphaLayer.enabled = false; alphaLayer.shy = true;
        var imageLayer = addLayer(output, rgb, 'Chromatic image');
        var setMatte = effect(imageLayer, 'ADBE Set Matte3', 'Restore transparency');
        setMatte.property(1).setValue(alphaLayer.index);
        setMatte.property(2).setValue(1); // Red (coverage is grayscale)
        setMatte.property(3).setValue(0);
        setMatte.property(4).setValue(0);
        setMatte.property(5).setValue(0);
        setMatte.property(6).setValue(0);
        // RGB was summed against black; unpremultiply once after restoring alpha.
        var unpremultiply = effect(imageLayer, 'ADBE Remove Color Matting', 'Clean transparent edges');
        unpremultiply.property(1).setValue([0, 0, 0, 1]);
        unpremultiply.property(2).setValue(0);
        var audioLayer = addLayer(output, source, 'Original audio');
        audioLayer.enabled = false; audioLayer.audioEnabled = true; audioLayer.shy = true;
        audioLayer.comment = TAG + ':source';
        // Effect layer indices are maintained by AE as layers move.
        ctl.moveToBeginning();
        imageLayer.shy = true; output.hideShyLayers = true;
        return { output: output, rgb: rgb, matte: matte, controller: ctl, source: source, folder: folder };
    }
    function contains(layers, target) {
        for (var i = 0; i < layers.length; i++) if (layers[i] === target) return true;
        return false;
    }
    function validate(comp, layers) {
        if (!(comp instanceof CompItem) || !layers.length) throw new Error('Open a composition and select one or more 2D layers first.');
        var min = comp.numLayers, max = 0;
        for (var i = 0; i < layers.length; i++) {
            var l = layers[i];
            if (!(l instanceof AVLayer) || !l.hasVideo || l.nullLayer || l.adjustmentLayer || l.threeDLayer || l.guideLayer)
                throw new Error('Select ordinary 2D footage, text, shapes, or a precomp. Precompose 3D scenes and adjustment layers first.');
            if (l.locked) throw new Error('Unlock the selected layers first.');
            if (l.parent && !contains(layers, l.parent)) throw new Error('Precompose this layer with its parent first, then apply Chromatic.');
            if (l.hasTrackMatte || l.isTrackMatte) throw new Error('Precompose the layer and its track matte first, then apply Chromatic.');
            if (l.blendingMode !== BlendingMode.NORMAL) throw new Error('Precompose layers using blending modes with their background first.');
            if (l.source instanceof CompItem && l.source.comment === TAG + ':output') throw new Error('This layer already has Chromatic. Use Open controls.');
            min = Math.min(min, l.index); max = Math.max(max, l.index);
        }
        if (max - min + 1 !== layers.length) throw new Error('Select adjacent layers, or precompose the group first.');
        for (i = 1; i <= comp.numLayers; i++) {
            if (comp.layer(i).parent && contains(layers, comp.layer(i).parent) && !contains(layers, comp.layer(i)))
                throw new Error('A selected layer has children outside the selection. Precompose the complete group first.');
        }
    }
    function apply(comp, layers, options) {
        validate(comp, layers);
        var indices = [], minIn = comp.duration, maxOut = 0, firstName = layers[0].name;
        for (var i = 0; i < layers.length; i++) {
            indices.push(layers[i].index);
            minIn = Math.min(minIn, layers[i].inPoint); maxOut = Math.max(maxOut, layers[i].outPoint);
        }
        indices.sort(function (a, b) { return a - b; });
        var source = comp.layers.precompose(indices, firstName + ' - Chromatic source', true);
        var wrapper;
        for (i = 1; i <= comp.numLayers; i++) if (comp.layer(i).source === source) { wrapper = comp.layer(i); break; }
        var rig = build(source, options);
        source.parentFolder = rig.folder;
        wrapper.replaceSource(rig.output, false);
        wrapper.name = firstName + ' - Chromatic'; wrapper.comment = TAG + ':instance';
        wrapper.startTime = 0; wrapper.inPoint = minIn; wrapper.outPoint = maxOut; wrapper.label = 14;
        rig.output.time = Math.max(0, Math.min(source.duration - source.frameDuration, comp.time));
        rig.wrapper = wrapper;
        return rig;
    }
    function findRig() {
        var c = app.project.activeItem;
        if (!(c instanceof CompItem)) throw new Error('Open a composition and select a Chromatic layer.');
        if (c.comment === TAG + ':output') return c;
        var selected = c.selectedLayers;
        if (selected.length === 1 && selected[0].source instanceof CompItem && selected[0].source.comment === TAG + ':output') {
            var rig = selected[0].source;
            var t = (c.time - selected[0].startTime) * 100 / selected[0].stretch;
            if (selected[0].timeRemapEnabled) t = selected[0].property('ADBE Time Remapping').valueAtTime(c.time, false);
            rig.time = Math.max(0, Math.min(rig.duration - rig.frameDuration, t));
            return rig;
        }
        throw new Error('Select a Chromatic layer, then click Open controls.');
    }
    function openControls(comp) {
        comp.openInViewer();
        for (var i = 1; i <= comp.numLayers; i++) comp.layer(i).selected = false;
        comp.layer(CONTROL).selected = true;
        app.executeCommand(2163); // Effect Controls, independent of UI language.
    }
    function readNumber(field, name, minimum) {
        var n = Number(field.text);
        if (field.text.replace(/\s/g, '') === '' || !isFinite(n) || (minimum !== undefined && n < minimum))
            throw new Error('Enter a valid ' + name + (minimum !== undefined ? ' of ' + minimum + ' or more.' : '.'));
        return n;
    }
    function ui(host) {
        var w = host instanceof Panel ? host : new Window('palette', 'Chromatic AE', undefined, { resizeable: true });
        w.orientation = 'column'; w.alignChildren = ['fill', 'top']; w.spacing = 10; w.margins = 16;
        var title = w.add('statictext', undefined, 'CHROMATIC AE');
        title.graphics.font = ScriptUI.newFont(title.graphics.font.name, 'BOLD', 20);
        w.add('statictext', undefined, 'Split. Recolour. Soften.');
        var box = w.add('panel', undefined, 'Starting look'); box.orientation = 'column'; box.alignChildren = ['fill', 'top']; box.margins = 12;
        var preset = box.add('dropdownlist', undefined, ['Classic RGB', 'Subtle RGB', 'Soft RGB', 'Green to teal']); preset.selection = 0;
        function field(label, value) {
            var row = box.add('group'); row.add('statictext', undefined, label).preferredSize.width = 145;
            var f = row.add('edittext', undefined, value); f.characters = 7; return f;
        }
        var distance = field('Split distance (px)', '8');
        var angle = field('Direction (degrees)', '0');
        var blur = field('Edge softness (px)', '0');
        preset.onChange = function () {
            var p = preset.selection.index;
            distance.text = p === 1 ? '2' : '8'; blur.text = p === 2 ? '2' : '0';
        };
        var create = w.add('button', undefined, 'Apply to selected layers'); create.preferredSize.height = 36;
        var row = w.add('group'); row.alignChildren = ['fill', 'center'];
        var edit = row.add('button', undefined, 'Open controls');
        var original = row.add('button', undefined, 'Edit source');
        var tip = w.add('statictext', undefined, 'Select layers and apply. Animate the control layer\'s effect stopwatches. Colours and softness are independent for R, G and B.', { multiline: true });
        tip.preferredSize = [310, 58];
        var status = w.add('statictext', undefined, 'Ready  |  v' + VERSION, { multiline: true }); status.preferredSize = [310, 38];
        create.onClick = function () {
            var started = false;
            try {
                var opts = { distance: readNumber(distance, 'split distance', 0), angle: readNumber(angle, 'direction'), blur: readNumber(blur, 'edge softness', 0), teal: preset.selection.index === 3 };
                var comp = app.project.activeItem;
                var layers = comp instanceof CompItem ? comp.selectedLayers : [];
                validate(comp, layers);
                app.beginUndoGroup('Apply Chromatic AE'); started = true;
                var rig = apply(comp, layers, opts);
                openControls(rig.output);
                status.text = 'Applied. Use the stopwatches in Effect Controls.\nYour original layers are in Edit source.';
            } catch (error) { alert('Chromatic AE\n\n' + error.toString() + (started ? '\n\nUse Edit > Undo to revert this operation.' : '')); }
            finally { if (started) app.endUndoGroup(); }
        };
        edit.onClick = function () { try { openControls(findRig()); } catch (e) { alert(e.toString()); } };
        original.onClick = function () {
            try {
                var rig = findRig();
                for (var i = 1; i <= rig.numLayers; i++) if (rig.layer(i).comment === TAG + ':source') {
                    rig.layer(i).source.time = rig.time; rig.layer(i).source.openInViewer(); return;
                }
            } catch (e) { alert(e.toString()); }
        };
        w.onResizing = w.onResize = function () { this.layout.resize(); };
        w.layout.layout(true); w.minimumSize = [340, 390];
        if (w instanceof Window) { w.center(); w.show(); }
        return w;
    }
    if (typeof module !== 'undefined' && module.exports) module.exports = { positionExpression: positionExpression, quote: quote };
    else if ($.global.CHROMATIC_AE_TEST_MODE) $.global.ChromaticAE = { build: build, apply: apply, validate: validate, ctrl: ctrl, ui: ui, openControls: openControls, version: VERSION };
    else ui(host);
}(this));
