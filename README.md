
	# Project ReactiveRS

Luc Chabassier et David Reboullet

## Structure de la librairie

La librairie est divisée en trois sous modules : le runtime, les processus (arrow) et les signaux.

Comme suggéré dans le sujet les runtimes manipulent des continuations. Néanmoins, "Runtime" est
ici un trait qui définit les méthodes essentielles de tout runtime, à savoir pouvoir appeler des
continuations à cet instant, à la fin de l'instant ou à l'instant suivant et pouvoir executer le
runtime jusqu'à qu'il n'y ait plus de travail. Deux implémentations de ce trait sont présentes :
le runtime séquentiel "SeqRuntime" et le runtime parallèle "ParRuntime". Le runtime parallèle a
été implémenté 'à la main' à partir de la librairie standard. A des fins d'optimisation,
l'instance d'un thread d'un runtime n'enregistre une continuation de cet instant sur la structure
partagée qu'à condition qu'il en ait déja une enregistrée en local.

Le module "arrow" ("src/arrow/mod.rs") définit l'alternative de notre projet aux processus. Il
s'agit d'une sorte de processus qui au lieu de simplement produire une valeur attend aussi une
valeur. En ce sens c'est un mélange des continuations et des processus du sujet. Ce choix a été
fait afin d'éviter la duplication de code du à la distinction entre "Process" et "ProcessMut" :
les 'arrows' ne sont pas détruits après utilisation. On a aussi remarqué que indépendemment de
ce fait les fonctions étaient plus simples avec cet interface, mais les signatures plus lourdes.

Les seules opérations définies dans ce module sont le 'bind', qui correspond à la composition de
fonctions, et le 'flatten', qui consiste à éxécuter le processus retourné par un autre processus.
Toutes les autres primitives sont implémentées dans le module "signal::prim"
("src/arrow/prim.rs"). Les primitives notables sont le 'Fork' qui lance l'éxécution
parallèle d'un autre processus, le 'Fixpoint' qui permet de réaliser des boucles, le 'Product' qui
parallèlisme l'éxécution de deux processus (construction || de ReactiveML) et le 'SeqProd' qui
fait la même chose mais séquentiellement. Un processus constant, un 'Map' et un 'Pause' sont
bien entendu présents. Les arrows peuvent être éxécuté sur un runtime préalablement construit ou
grâce à deux méthodes ("execute_seq") et ("execute_par") qui gèrent elles-mêmes le runtime.

Le module "signal" ("src/signal/mod.rs") définit le trait "Signal". Ce trait décrit un objet
disposant de méthodes permettant d'implémenter la structure "await immediate" et "present" de
ReactiveML. Le module "signal::prim" ("src/signal/prim.rs") implémente quand à lui trois types de
signaux : les signaux pures ("PureSignal") ne demandant pas de valeurs, les signaux normaux (
"ValueSignal") correspondant aux signaux de ReactiveML, et les signaux à consommateur unique (
"UniqSignal") qui contrairement aux signaux normaux ne demandent pas que le type des valeurs
implémentent le trait "Clone". Ceci est possible car contrairement aux "ValueSignal", les
"UniqSignal", fournissent l'unique processus consommateur lors de leur création et ne permettent
plus ensuite de le cloner ou d'en générer un autre.

Afin de simplifier l'écriture de processus, la macro arrow! permet d'écrire du code plus proche
de ce qui se ferait en reactiveML. Seul le système de macro simple de Rust a été utilisé pour
l'implémenter, illustrant la puissance de ce système.

## Le programme

Comme programme on a voulu réimplémenter le pacman du TD de reactiveML avec notre librairie. Des
difficultés se sont posées, qui illustrent les limites de notre librairie et/ou de rust.

Le premier problème qui s'est posé est l'absence d'inférence de type pour les fonctions. Ceci
signifie que si l'on crée une fonction qui retourne un processus, on doit nécessairement mettre le
type entier dans la signature de la fonction, ce qui est problématique puisque son type encode
la structure du processus, et est donc long, illisible, et doit être changé chaque fois que le
processus évolue. Afin de contourner ce problème, tous les processus sont implémentés dans des
variables locales de la fonction main, qui elles bénéficient de l'inférence de type.

Le second problème est que pour supporter le runtime parallèle, tous nos processus doivent avoir
le trait Sync, même si seulement le runtime séquentiel va être utilisé. Or les fonctions
d'affichage de piston ne l'implémentent pas. Il a donc du fallu bricoller quelque chose à base de
threads et de mpsc pour gérer l'affichage. De manière générale il semble difficile d'interfacer
correctement du code réactif utilisant notre librairie avec du code d'autres librairies.

Ces complications ont ralenties l'écriture du pacman qui pour l'instant ne supporte que le
déplacement du pacman seul, sans animation. Cependant cela suffit à montrer l'expressivité de notre
système et ses limites.

