# Which Codepoint

Find out which fonts support your codepoints.

## Quickwit

I provide a config for quickwit indexes in the `indexes/` folder. You can use that to set up and index fonts.
Searching can be done using quickwit as well.


## Example

The `cpdump` tool is meant to dump indexes to stdout for a single font file at a time. It uses the quickwit index format, meaning a series of json objects delimited by newline.

Using nushell, I set up with the following commands:

```nu
let uri = ($"file://(pwd)/indexes/mappings");
let font_index = indexes/fonts;

# Create a new index
quickwit new --index-uri $uri --index-config-path ./indexes/quickwit_config.json;

# Create index json file
ls path/to/fonts | each { cpdump $it.name } | save --raw $font_index

# Index using quickwit
quickwit index --index-uri $uri --input-path $font_index
```

After that, you can get the list of fonts that support a given code point (e.g. `U+0001`) like so:

```nu
λ quickwit search --index-uri $"file://(pwd)/indexes/mappings" --query "0001" | from json | get hits | get font_name
   0   ABeeZee Italic                          
   1   Amita Regular                           
   2   Arbutus Slab Regular                    
   3   Armata Regular                          
   4   Arya Bold                               
   5   Arya Regular                            
   6   Carrois Gothic Regular                  
   7   Denk One                                
   8   Donegal One                             
   9   Duru Sans Regular                       
  10   Fruktur                                 
  11   Habibi                                  
  12   Hammersmith One Regular                 
  13   Inder                                   
  14   Libre Barcode 39 Extended Text Regular  
  15   Metamorphous                            
  16   Molle                                   
  17   Oldenburg                               
  18   Passero One                             
  19   Pompiere
```