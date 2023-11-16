import json 
with open ('mydata.csv', 'r') as f: 
    loaded_json = json.load(f)
anarray = []
for value in loaded_json['result']:
    anarray.append(value['pubkey'])
with open ('mydata.json', 'w') as f: 
    f.write(json.dumps(anarray))