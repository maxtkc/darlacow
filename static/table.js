var sequence="loading...";
function send_data(data) {
    if(data == undefined) {
        data = table.getData();
    }
    console.log(data);
    $.ajax("set/" + sequence, {method: "POST", format: "application/json", data: JSON.stringify(data)});
    $("#note").html("Autosaved at " + new Date());
}

function refresh() {
    if(sequence != "loading...") {
        table.setData("get/" + sequence);
    }
    $.ajax({url: "get-seqs", success: function(result) {
            $("#seq").html("");
            var names = JSON.parse(result);
            if(sequence == "loading...") {
                sequence = names[0];
            }
            for(i in names) {
                if(names[i] == sequence) {
                    $("#seq").append("<option value=\"" + names[i] + "\" selected>" + names[i] + "</option>");
                } else {
                    $("#seq").append("<option value=\"" + names[i] + "\">" + names[i] + "</option>");
                }
            }
            }});
}

var table = new Tabulator("#edit-table", {
    dataEdited:function(data){
    //data - the updated table data
        send_data(data);
    },
height:"100%",
layout:"fitColumns",
resizableColumns:false,
columns:[
{title:"Time", field:"time", editor:"number", headerVertical:true, align:"center"},
{title:"Song", field:"song", editor:"select", headerVertical:true, editorParams:{values:{ {% for song in songs %}"{{ song }}":"{{ song }}"{% if not loop.last %},{% endif %}{% endfor %} }}, align:"center"},
{% for name in relays %}
{title:"{{name}}", field:"{{name}}", headerVertical:true, editor:"tickCross", formatter:"tickCross", editorParams:{indeterminateValue:"n/a"}, align:"center"},
{% endfor %}
{title:"Secondary<br>Motion", field:"sec_mot", editor:"select", headerVertical:true, editorParams:{values:{"none":"none", "in":"in", "out":"out"}}, align:"center"},
],
});
refresh();

//Add row on "Add Row" button click
$("#add-row").click(function(){
    var tar = table.getRows().length - 1;
    if (tar < 0) {
        table.addRow({time:1000, sec_mot:"none"});
    }
    else {
        table.addRow(table.getRowFromPosition(tar)._row.data);
    }
});

//Delete row on "Delete Row" button click
$("#del-row").click(function(){
    table.deleteRow(table.getRowFromPosition(table.getRows().length - 1));
});

//Clear table on "Empty the table" button click
$("#clear").click(function(){
    table.clearData();
});

//Reset table contents on "Reset the table" button click
$("#save").click(function(){
    send_data();
});

$("#seq").change(function(){
    sequence=$("#seq").val();
    refresh();
});

$("#new-seq").click(function(){
    sequence=window.prompt("What would you like to name your new sequence?", "");
    $.ajax({url: "/new-seq/" + sequence, success: function(result) {
        refresh();
        }});
});
