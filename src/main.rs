// ============================================================================
// MOTOR DE TRÁFICO AÉREO - ÁRBOL AVL
// ============================================================================

#[derive(Debug, Clone)]
struct Vuelo {
    id: String,
    altitud: u32, // Clave principal del árbol
}

#[derive(Debug)]
struct Nodo {
    vuelo: Vuelo,
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    /// Crea un nodo nuevo sin hijos.
    /// La altura inicial es 1 porque el nodo ya cuenta como una hoja.
    fn nuevo(vuelo: Vuelo) -> Self {
        Nodo {
            vuelo,
            izquierdo: None,
            derecho: None,
            altura: 1,
        }
    }
}

// ============================================================================
// UTILIDADES AVL
// ============================================================================

/// Devuelve la altura de un nodo.
/// as_ref() presta una referencia sin mover la propiedad.
/// Si el nodo no existe, devuelve 0.
fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

/// Recalcula la altura del nodo usando la altura de sus hijos.
fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

/// Factor de balance = altura_izquierda - altura_derecha.
/// Si es mayor que 1, el árbol está cargado a la izquierda.
/// Si es menor que -1, está cargado a la derecha.
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

// ============================================================================
// FASE 1 - MEMORIA Y ROTACIONES
// ============================================================================
//
// ¿Por qué se usa .take()?
// .take() saca el valor de un Option y deja None en su lugar.
// Eso permite mover el hijo de un nodo sin romper ownership.
// Así Rust no se queja con el Borrow Checker y no hace falta usar clone().
//
// ¿Por qué se usa Box<Nodo>?
// Porque un Nodo contiene otros Nod​os como hijos.
// Si se guardara Nodo directamente dentro de Nodo, el tamaño sería infinito.
// Box guarda el nodo en el heap y deja dentro del struct solo un puntero de
// tamaño fijo. Eso hace posible que el árbol exista en Rust de forma segura.
//

/// Rotación simple a la derecha.
/// Se usa cuando el árbol está cargado a la izquierda.
fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    // .take() mueve el hijo izquierdo de y hacia x, dejando None en y.izquierdo
    let mut x = y.izquierdo.take().expect("Hijo izquierdo ausente");

    // El subárbol derecho de x pasa a ser el hijo izquierdo de y
    y.izquierdo = x.derecho.take();

    // Primero actualizamos y porque cambió su posición
    actualizar_altura(&mut y);

    // y pasa a ser hijo derecho de x
    x.derecho = Some(y);

    // Luego actualizamos x, que ahora es la nueva raíz
    actualizar_altura(&mut x);

    x
}

/// Rotación simple a la izquierda.
/// Se usa cuando el árbol está cargado a la derecha.
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Hijo derecho ausente");
    x.derecho = y.izquierdo.take();
    actualizar_altura(&mut x);
    y.izquierdo = Some(x);
    actualizar_altura(&mut y);
    y
}

/// Balancea un nodo después de insertar o eliminar.
/// Resuelve los cuatro casos clásicos: LL, RR, LR y RL.
fn balancear(mut nodo: Box<Nodo>) -> Box<Nodo> {
    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso LL: pesado a la izquierda y el hijo izquierdo también a la izquierda
    if balance > 1 {
        if obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {
            let hijo_izq = nodo.izquierdo.take().unwrap();
            nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        }
        return rotar_derecha(nodo);
    }

    // Caso RR: pesado a la derecha y el hijo derecho también a la derecha
    if balance < -1 {
        if obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {
            let hijo_der = nodo.derecho.take().unwrap();
            nodo.derecho = Some(rotar_derecha(hijo_der));
        }
        return rotar_izquierda(nodo);
    }

    nodo
}

// ============================================================================
// FASE 2 - INSERCIÓN
// ============================================================================

/// Inserta un vuelo según su altitud.
/// No permite duplicados de altitud.
/// Retorna la nueva raíz balanceada.
fn insertar(nodo_opt: Option<Box<Nodo>>, vuelo: Vuelo) -> Box<Nodo> {
    let clave = vuelo.altitud;

    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(vuelo)),
        Some(n) => n,
    };

    if clave < nodo.vuelo.altitud {
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), vuelo));
    } else if clave > nodo.vuelo.altitud {
        nodo.derecho = Some(insertar(nodo.derecho.take(), vuelo));
    } else {
        // Si la altitud ya existe, no se inserta otra vez.
        return nodo;
    }

    balancear(nodo)
}

// ============================================================================
// FASE 3 - BÚSQUEDA
// ============================================================================

/// Busca un vuelo por altitud.
/// Solo usa referencias, por eso no modifica el árbol.
/// La búsqueda es O(log n) si el árbol está balanceado.
fn buscar_vuelo(nodo: &Option<Box<Nodo>>, altitud: u32) -> Option<&Vuelo> {
    match nodo.as_ref() {
        None => None,
        Some(n) => {
            if altitud < n.vuelo.altitud {
                buscar_vuelo(&n.izquierdo, altitud)
            } else if altitud > n.vuelo.altitud {
                buscar_vuelo(&n.derecho, altitud)
            } else {
                Some(&n.vuelo)
            }
        }
    }
}

// ============================================================================
// FASE 4 - ELIMINACIÓN
// ============================================================================
//
// La eliminación debe manejar 3 casos:
// 1. Nodo sin hijos.
// 2. Nodo con un hijo.
// 3. Nodo con dos hijos.
//
// Si tiene dos hijos, se usa el predecesor in-order:
// el valor más alto del subárbol izquierdo.
// Luego se actualizan alturas y se rebalancea.
//

/// Extrae el valor más alto de un subárbol.
/// Ese valor sirve como predecesor in-order.
/// Devuelve solo el Vuelo para evitar copias innecesarias.
fn extraer_maximo(subarbol: &mut Option<Box<Nodo>>) -> Vuelo {
    let mut nodo = subarbol.take().expect("Subárbol vacío");

    // Si no hay hijo derecho, este nodo es el máximo.
    if nodo.derecho.is_none() {
        let izq = nodo.izquierdo.take();
        let vuelo = nodo.vuelo;
        *subarbol = izq;
        vuelo
    } else {
        // Si sí hay hijo derecho, el máximo está más a la derecha.
        let max_vuelo = extraer_maximo(&mut nodo.derecho);
        nodo = balancear(nodo);
        *subarbol = Some(nodo);
        max_vuelo
    }
}

/// Elimina un vuelo por altitud.
/// Retorna la nueva raíz del subárbol.
fn eliminar_vuelo(nodo_opt: Option<Box<Nodo>>, altitud: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None,
        Some(n) => n,
    };

    if altitud < nodo.vuelo.altitud {
        nodo.izquierdo = eliminar_vuelo(nodo.izquierdo.take(), altitud);
        return Some(balancear(nodo));
    }

    if altitud > nodo.vuelo.altitud {
        nodo.derecho = eliminar_vuelo(nodo.derecho.take(), altitud);
        return Some(balancear(nodo));
    }

    // Caso 1: sin hijos
    if nodo.izquierdo.is_none() && nodo.derecho.is_none() {
        return None;
    }

    // Caso 2: un solo hijo
    if nodo.izquierdo.is_none() {
        return nodo.derecho;
    }
    if nodo.derecho.is_none() {
        return nodo.izquierdo;
    }

    // Caso 3: dos hijos
    // Se reemplaza el nodo con el predecesor in-order:
    // el valor más alto del subárbol izquierdo.
    let predecesor = extraer_maximo(&mut nodo.izquierdo);
    nodo.vuelo = predecesor;

    Some(balancear(nodo))
}

// ============================================================================
// FASE 5 - ALERTA DE PROXIMIDAD
// ============================================================================

/// Cuenta cuántos vuelos están dentro de un rango de altitud.
/// Solo recorre las ramas que pueden tener valores válidos.
fn vuelos_en_rango(nodo: &Option<Box<Nodo>>, min: u32, max: u32) -> usize {
    match nodo.as_ref() {
        None => 0,
        Some(n) => {
            let mut total = 0;

            if n.vuelo.altitud >= min && n.vuelo.altitud <= max {
                total += 1;
            }

            if n.vuelo.altitud > min {
                total += vuelos_en_rango(&n.izquierdo, min, max);
            }

            if n.vuelo.altitud < max {
                total += vuelos_en_rango(&n.derecho, min, max);
            }

            total
        }
    }
}

// ============================================================================
// RECORRIDO VISUAL PARA VERIFICAR EL ÁRBOL
// ============================================================================

/// Imprime el árbol girado 90 grados.
/// Sirve para comprobar fácilmente si el balance quedó bien.
fn imprimir_arbol(nodo: &Option<Box<Nodo>>, nivel: usize) {
    if let Some(n) = nodo.as_ref() {
        imprimir_arbol(&n.derecho, nivel + 1);
        println!(
            "{:indent$}[{} - {}] (h={})",
            "",
            n.vuelo.id,
            n.vuelo.altitud,
            n.altura,
            indent = nivel * 4
        );
        imprimir_arbol(&n.izquierdo, nivel + 1);
    }
}

// ============================================================================
// MAIN
// ============================================================================

fn main() {
    let mut radar: Option<Box<Nodo>> = None;

    // Datos pedidos por el documento.
    let datos = vec![
        ("AV123", 5000),
        ("UA456", 3000),
        ("IB101", 2000),
        ("AF999", 4000),
        ("TA222", 3500),
        ("AM777", 6000),
    ];

    for (id, alt) in datos {
        let vuelo = Vuelo {
            id: id.to_string(),
            altitud: alt,
        };
        radar = Some(insertar(radar.take(), vuelo));
    }

    println!("=== RADAR AVL INICIAL ===");
    imprimir_arbol(&radar, 0);

    // Búsqueda de prueba
    println!("\n=== BÚSQUEDA ===");
    match buscar_vuelo(&radar, 3500) {
        Some(v) => println!("Encontrado: {} en altitud {}", v.id, v.altitud),
        None => println!("No encontrado"),
    }

    // Eliminación de prueba
    println!("\n=== ELIMINACIÓN DE 3000 ===");
    radar = eliminar_vuelo(radar, 3000);
    imprimir_arbol(&radar, 0);

    // Función extra de proximidad
    println!(
        "\nVuelos entre 3000 y 5000: {}",
        vuelos_en_rango(&radar, 3000, 5000)
    );

// ========================================================================
// PRUEBA DE ESCRITORIO (ALINEADA CON LA EJECUCIÓN DEL PROGRAMA)
// Inserciones: [5000, 3000, 2000, 4000, 3500, 6000]
//
// 1) Insertar 5000:
//    - Se convierte en la raíz.
//
// 2) Insertar 3000:
//    - Va a la izquierda de 5000.
//    - No hay desbalance.
//
// 3) Insertar 2000:
//    - Se genera desbalance en 5000 (caso LL).
//    - Se aplica rotación simple a la derecha.
//    - Nuevo árbol:
//          3000
//         /    \
//      2000   5000
//
// 4) Insertar 4000:
//    - Va como hijo izquierdo de 5000.
//    - El árbol sigue balanceado.
//
// 5) Insertar 3500:
//    - Se inserta como hijo izquierdo de 4000.
//    - Se genera un desbalance tipo LR en el nodo 3000.
//    - Se aplica rotación doble:
//       1. Rotación izquierda en el hijo izquierdo (3000 → 4000)
//       2. Rotación derecha en la raíz (3000)
//    - Nueva raíz: 4000   
//        
// 6) Insertar 6000:
//    - Va como hijo derecho de 5000.
//    - El árbol permanece balanceado.
//
// Árbol final obtenido:
//
//              4000
//            /      \
//         3000      5000
//        /   \         \
//      2000  3500      6000
//
// Todos los factores de balance están entre -1 y 1.
// El árbol cumple completamente la propiedad AVL.
// ========================================================================
}
