import random

_nm1 = ["a","e","i","u","o","a","ai","aiu","aiue","e","i","ia","iau","iu","o","u","y","ya","yi","yo"];
_nm2 = ["bh","br","c'th","cn","ct","cth","cx","d","d'","g","gh","ghr","gr","h","k","kh","kth","mh","mh'","ml","n","ng","sh","t","th","tr","v","v'","vh","vh'","vr","x","z","z'","zh"];
_nm3 = ["a","e","i","u","o","a","e","i","u","o","ao","aio","ui","aa","io","ou","y"];
_nm4 = ["bb","bh","br","cn","ct","dh","dhr","dr","drr","g","gd","gg","ggd","gh","gn","gnn","gr","jh","kl","l","ld","lk","ll","lp","lth","mbr","nd","p","r","rr","rv","th","thl","thr","thrh","tl","vh","x","xh","z","zh","zt"];
_nm5 = ["","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","'dhr","'dr","'end","'gn","'ith","'itr","'k","'kr","'l","'m","'r","'th","'vh","'x","'zh"];
_nm6 = ["a","e","i","u","o"];
_nm7 = ["","","","","","","","","","","d","g","h","l","lb","lbh","n","r","rc","rh","s","sh","ss","st","sz","th","tl","x","xr","xz"];
_br = "";

def randomName():
    if random.random() > 5./15.:
        names  = random.choice(_nm1)
    else:
        names  = ""

    names += random.choice(_nm2)
    names += random.choice(_nm3)
    names += random.choice(_nm4)
    names += random.choice(_nm5)
    names += random.choice(_nm6)
    names += random.choice(_nm7)
    return names
