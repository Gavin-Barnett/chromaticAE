(function(){
    var root=new File($.fileName).parent.parent.fsName;
    $.evalFile(new File(root+'/tests/native-integration.jsx'));
    $.evalFile(new File(root+'/tests/native-bounds.jsx'));
}());
