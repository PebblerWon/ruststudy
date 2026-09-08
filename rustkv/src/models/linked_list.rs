#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub value: String,
    pub next: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq, Default)]

pub struct LinkedList {
    head: Option<Box<Node>>,
    len: usize,
}

impl LinkedList {
    pub fn new() -> Self {
        LinkedList { head: None, len: 0 }
    }

    pub fn push(&mut self, value: String) -> usize {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(),
        });
        self.head = Some(new_node);
        self.len += 1;
        self.len
    }
    pub fn pop(&mut self) -> Option<String> {
        match self.head.take() {
            Some(n) => {
                self.head = n.next;
                self.len -= 1;
                Some(n.value)
            }
            None => None,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0 || self.head.is_none()
    }
    pub fn from_vec(items: &[&str]) -> Self {
        let mut list = LinkedList::new();

        for n in items.iter().rev() {
            list.push(n.to_string());
        }
        list
    }
    pub fn push2(&mut self, value: String) -> usize {
        let n = Box::new(Node { value, next: None });

        if self.head.is_none() {
            self.head = Some(n);
            self.len += 1;
            self.len
        } else {
            let mut tail = self.head.as_mut();

            while let Some(node) = tail {
                let next = &node.next;
                if (next.is_none()) {
                    node.next = Some(n);
                    self.len += 1;
                    return self.len;
                } else {
                    tail = node.next.as_mut();
                }
            }
            0
        }
    }
}

impl std::fmt::Display for LinkedList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::<&str>::new();
        let mut next = &self.head;
        while let Some(node) = next {
            v.push(&node.value);
            next = &node.next;
        }
        write!(f, "{}", v.join(" "))
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    pub fn testa() {
        let mut a = LinkedList::new();
        a.push("1".to_string());
        let n = a.pop();
        assert!(n.is_some());
        assert_eq!(n.unwrap(), "1");
        assert_eq!(a.len(), 0);
        let n2 = a.pop();
        assert!(n2.is_none());
        assert_eq!(a.len(), 0);
    }
}
