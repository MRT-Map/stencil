/// <reference path="references.ts" />
function triggerImportData() {
    qs(document, "#importerComps").click();
    qs(document, "#importerNodes").click();
}
function importData(id) {
    var importedFile = qs(document, '#' + id).files[0];
    var reader = new FileReader();
    reader.onload = function () {
        var fileContent = JSON.parse(reader.result);
        document.getElementById('importer').value = "";
        //console.log(fileContent);
        if (id == "importerComps") {
        }
        else if (id == "importerNodes") {
        }
    };
    reader.readAsText(importedFile);
}
