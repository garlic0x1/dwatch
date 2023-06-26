# NOTE
For a simpler alternative you can use this version https://github.com/garlic0x1/dwatch-2  
  
# dwatch
Monitor directories for file changes and run scripts or restart servers, 
dwatch looks for a `.dwatch` file by default but you can specify one with `dwatch -f dwatch.yaml`

# examples
Command:  
```dwatch -f example.dwatch```  

Config:  
#### **`example.dwatch`**
```yaml
---
# example golang backend
- dir: ./
  filetypes: ["go"]
  servers:
    - go run . -p 8000

# example react build
- dir: ./src
  scripts:
    - npm run build

# example scss to css
- dir: ./scss
  scripts:
    - sass --update scss:.

# example backend restart
- dir: ./src
  filetypes: ["rs"]
  servers: ["cargo run -- --port 5000"]
  delay: 5

# or use json:
# [
#   {
#     "dir": "./src",
#     "filetypes": ["js", "jsx"],
#     "scripts": ["npm run build"]
#   }
# ]
```
