(defun fact (n)
  "The factorial function"
  (if (= n 0)
      1
      (* n
         (fact (- n 1)))))
