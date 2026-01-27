rm -recurse .\dist\
mkdir .\dist
cp .\target\release\todo.exe .\dist\
winapp pack .\dist\
