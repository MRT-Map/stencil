/// <reference path="references.ts" />

function triggerImportData() {
    qs(document, "#importerComps").click();
    qs(document, "#importerNodes").click();
}

function importData(id: string) {
    var importedFile = (qs(document, '#'+id) as HTMLInputElement).files[0];

    var reader = new FileReader();
    reader.onload = function() {
        var fileContent = JSON.parse(reader.result as string);
        (document.getElementById('importer') as HTMLInputElement).value = "";
        //console.log(fileContent);
        if (id == "importerComps") {
            
        } else if (id == "importerNodes") {

        }
    };
    reader.readAsText(importedFile); 
}