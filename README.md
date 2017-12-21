
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
l'instance d'un thread d'un runtime n'enregistre un continuation de cet instant sur la structure
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
plus ensuite de la cloner ou dans générer un autre.

